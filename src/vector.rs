use std::ptr::*;
use std::mem;
use std::alloc::{self, Layout};
use std::ops::Deref;
use std::ops::DerefMut;

// -------------- structs
pub struct Vector<T> {
  buffer: RawVector<T>,
  len: usize,
}

pub struct RawVector<T> {
  ptr: NonNull<T>,
  cap: usize,
}

pub struct IntoIter<T> {
  _buffer: RawVector<T>,
  start: *const T,
  end: *const T,
}

// -------------- impls
impl<T> Vector<T> {
  fn ptr(&self) -> *mut T {
    self.buffer.ptr.as_ptr()
  }

  fn cap(&self) -> usize {
    self.buffer.cap
  }

  pub fn new() -> Self {
    Vector {
      buffer: RawVector::new(),
      len: 0,
    }
  }

  pub fn push(&mut self, value: T) {
    // if we are full -> grow the buffer
    if self.len == self.cap() {
      self.buffer.grow();
    }

    unsafe {
      // writes to memory without reading or dropping the old value
      // .add() is the usize version of .offset() (isize)
      // goes from self.ptr to the next free slot at position self.len
      std::ptr::write(self.ptr().add(self.len), value);
    }
    
    self.len += 1;
  }

  pub fn pop(&mut self) -> Option<T> {
    if self.len == 0 {
      None
    } else {
      self.len -= 1;
      unsafe { Some(std::ptr::read(self.ptr().add(self.len))) }
    }
  }

  pub fn insert(&mut self, idx: usize, value: T) {
    assert!(idx <= self.len, "index out of bounds");

    if self.len == self.cap() {
      self.buffer.grow();
    }

    unsafe {
      std::ptr::copy(
          self.ptr().add(idx),
          self.ptr().add(idx + 1),
          self.len - idx,
      );

      std::ptr::write(self.ptr().add(idx), value);
    }

    self.len += 1;
  }

  pub fn remove(&mut self, idx: usize) -> T {
    assert!(idx < self.len, "index out of bounds");

    unsafe {
      self.len -= 1;

      let res = std::ptr::read(self.ptr().add(idx));

      std::ptr::copy(
        self.ptr().add(idx + 1),
        self.ptr().add(idx),
        self.len - idx,
      );

      return res;
    }
  }

  pub fn len(&self) -> usize {
    self.len
  }
}

impl<T> Drop for Vector<T> {
  fn drop(&mut self) {
    while let Some(_) = self.pop() {}
  }
}

// read-only view &[10, 20, 30] -> v.first(), v.iter()
impl<T> Deref for Vector<T> {
  type Target = [T];

  fn deref(&self) -> &[T] {
    unsafe {
      std::slice::from_raw_parts(self.ptr(), self.len)
    }
  }
}

// mutable view &mut [10, 20, 30] -> v[0] = 100;
impl<T> DerefMut for Vector<T> {
  fn deref_mut(&mut self) -> &mut [T] {
    unsafe {
      std::slice::from_raw_parts_mut(self.ptr(), self.len)
    }
  }
}

impl<T> IntoIterator for Vector<T> {
  type Item = T;
  type IntoIter = IntoIter<T>;

  fn into_iter(self) -> IntoIter<T> {
    let buf = unsafe { std::ptr::read(&self.buffer) };
    let len = self.len;

    mem::forget(self);

    IntoIter {
      start: buf.ptr.as_ptr(),

      end: if buf.cap == 0 {
        buf.ptr.as_ptr()
      } else {
        unsafe { buf.ptr.as_ptr().add(len) }
      },

      _buffer: buf,
    }
  }
}

// iterating forward
impl<T> Iterator for IntoIter<T> {
  type Item = T;

  fn next(&mut self) -> Option<T> {
    if self.start == self.end {
      None
    } else {
      unsafe {
        let res = std::ptr::read(self.start);
        self.start = self.start.offset(1);
        Some(res)
      }
    }
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    let len =
      (self.end as usize - self.start as usize) / mem::size_of::<T>();

    return (len, Some(len));
  }
}

// iterating backwards
impl<T> DoubleEndedIterator for IntoIter<T> {
  fn next_back(&mut self) -> Option<Self::Item> {
    if self.start == self.end {
      None
    } else {
      unsafe {
        self.end = self.end.offset(-1);
        Some(std::ptr::read(self.end))
      }
    }
  }
}

// IntoIter takes ownership of its allocation -> drop is needed to free it
impl<T> Drop for IntoIter<T> {
  fn drop(&mut self) {
    for _ in &mut *self {}
  }
}

impl<T> RawVector<T> {
  fn new() -> Self {
    assert!(
      mem::size_of::<T>() != 0,
      "TODO: implement ZST support"
    );

    RawVector {
      ptr: NonNull::dangling(),
      cap: 0,
    }
  }

  fn grow(&mut self) {
    let new_cap = if self.cap == 0 {
      1
    } else {
      2 * self.cap
    };

    let new_layout = Layout::array::<T>(new_cap).unwrap();

    assert!(
      new_layout.size() <= isize::MAX as usize,
      "Allocation too large"
    );

    let new_ptr = if self.cap == 0 {
      unsafe { alloc::alloc(new_layout) }
    } else {
      let old_layout = Layout::array::<T>(self.cap).unwrap();
      let old_ptr = self.ptr.as_ptr() as *mut u8;

      unsafe {
        alloc::realloc(
          old_ptr,
          old_layout,
          new_layout.size(),
        )
      }
    };

    self.ptr = match NonNull::new(new_ptr as *mut T) {
      Some(p) => p,
      None => alloc::handle_alloc_error(new_layout),
    };

    self.cap = new_cap;
  }
}

impl<T> Drop for RawVector<T> {
  fn drop(&mut self) {
    if self.cap != 0 {
      let layout = Layout::array::<T>(self.cap).unwrap();

      unsafe {
        alloc::dealloc(
          self.ptr.as_ptr() as *mut u8,
          layout,
        );
      }
    }
  }
}

// -------------- unsafe impls

unsafe impl<T: Send> Send for Vector<T> {}
unsafe impl<T: Sync> Sync for Vector<T> {}

unsafe impl<T: Send> Send for RawVector<T> {}
unsafe impl<T: Sync> Sync for RawVector<T> {}

// -------------- tests

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn push_and_pop() {
    let mut v = Vector::new();

    v.push(10);
    v.push(20);
    v.push(30);

    assert_eq!(v.len(), 3);

    assert_eq!(v.pop(), Some(30));
    assert_eq!(v.pop(), Some(20));
    assert_eq!(v.pop(), Some(10));
    assert_eq!(v.pop(), None);
  }

  #[test]
  fn insert() {
    let mut v = Vector::new();

    v.push(10);
    v.push(30);

    v.insert(1, 20);

    assert_eq!(v[0], 10);
    assert_eq!(v[1], 20);
    assert_eq!(v[2], 30);
  }

  #[test]
  fn remove() {
    let mut v = Vector::new();

    v.push(10);
    v.push(20);
    v.push(30);

    assert_eq!(v.remove(1), 20);

    assert_eq!(v[0], 10);
    assert_eq!(v[1], 30);
  }

  #[test]
  fn into_iter() {
    let mut v = Vector::new();

    v.push(10);
    v.push(20);
    v.push(30);

    let values: Vec<_> = v.into_iter().collect();

    assert_eq!(values, vec![10, 20, 30]);
  }
}