use std::mem::ManuallyDrop;
use std::ptr::*; 
use std::mem;
use std::alloc::{self, Layout};
use std::ops::Deref;
use std::ops::DerefMut;

pub struct Vector<T> {
  ptr: NonNull<T>, 
  cap: usize,
  len: usize,
}

impl<T> Vector<T>{
  pub fn new() -> Vector<T>{
    assert!(mem::size_of::<T>() != 0, "we're not ready to handle zero-sized types");
    Vector { 
      ptr: NonNull::dangling(), 
      cap: 0, 
      len: 0, 
    }
  }

  // el vector se quedo sin capacidad y pide mas memoria
  fn grow(&mut self){
    // calcular nueva capacidad (pido el doble)
    let(new_cap, new_layout) = 
      if self.cap == 0 {
        (1, Layout::array::<T>(1))
      }
      else{
        let new_cap = 2*self.cap;
        (new_cap, Layout::array::<T>(new_cap))
      };
    
    let new_layout = new_layout.expect("allocation too large!"); 
    // elegir entre alloc (primera vez) o realloc (crecer)
    let new_ptr = 
      if self.cap == 0{
        unsafe {alloc::alloc(new_layout)}
      }
      else{
        let old_layout = Layout::array::<T>(self.cap).unwrap();
        let old_ptr = self.ptr.as_ptr() as *mut u8;
        // para poder reubicar le paso el ptr viejo, el layout viejo y el tam. nuevo, asi copia los datos
        // y mueve la alocacion
        unsafe { alloc::realloc(old_ptr, old_layout, new_layout.size()) }
      };
    
    // si la alocacion fallo entonces el nuevo ptr es null, abortamos
    self.ptr = match NonNull::new(new_ptr as *mut T) {
      Some(p) => p, 
      None => alloc::handle_alloc_error(new_layout),
    };

    self.cap = new_cap
  }

  pub fn push(&mut self, value:T){
    // si estamos full -> expando
    if self.len == self.cap{
      self.grow();
    }

    unsafe{
      // escribe en memoria sin leer ni tirar el valor antiguo
      // .add() es la version usize de .offset() (isize)
      // va desde el self.ptr hasta la prox. pos. del slot libre (self.len) 
      std::ptr::write(self.ptr.as_ptr().add(self.len), value);
    }
  }

  pub fn pop(&mut self) -> Option<T>{
    if self.len == 0 {
      None
    }
    else{
      self.len -= 1;
      unsafe { Some(std::ptr::read(self.ptr.as_ptr().add(self.len))) }
    }
  }

  pub fn insert(&mut self, idx:usize, value:T){
    assert!(idx <= self.len, "index out of bounds");
    if self.len == self.cap{
      self.grow();
    }

    unsafe{
      // ptr::copy(src, dest, len) -> copy from src to dest len elems
      std::ptr::copy(
        self.ptr.as_ptr().add(idx), 
        self.ptr.as_ptr().add(idx + 1), 
        self.len - idx,
      )
    }

    self.len += 1
  }

  pub fn remove(&mut self, idx:usize) -> T {
    assert!(idx < self.len, "index out of bounds");
    unsafe {
      self.len -= 1;
      let res = std::ptr::read(self.ptr.as_ptr().add(idx)); 
      std::ptr::copy(
        self.ptr.as_ptr().add(idx+1),
        self.ptr.as_ptr().add(idx),
        self.len - idx,
      );
      return res;
    }
  }
} 

impl<T> Drop for Vector<T>{
  fn drop(&mut self) {
      if self.cap != 0 {
        while let Some(_) = self.pop() { }
        let layout = Layout::array::<T>(self.cap).unwrap();
        unsafe { alloc::dealloc(self.ptr.as_ptr() as *mut u8, layout);} 
      }
  }
}

// vista de solo lectura &[10,20,30] -> v.first(), v.iter()
impl<T> Deref for Vector<T>{
  type Target = [T];
  
  fn deref(&self) -> &[T]{
    unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len)}
  }
}

// vista modificable &mut [10,20,30] -> v[0] = 100;
impl<T> DerefMut for Vector<T>{
  fn deref_mut(&mut self) -> &mut [T]{
    unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len)}
  }
}

unsafe impl<T: Send> Send for Vector<T> {}
unsafe impl<T: Sync> Sync for Vector<T> {}


pub struct IntoIter<T>{
  buf:NonNull<T>, 
  cap:usize,
  start:*const T, 
  end:*const T, 
}

impl<T> IntoIterator for Vector<T>{
  type Item = T; 
  type IntoIter = IntoIter<T>;

  fn into_iter(self) -> IntoIter<T>{
    let vec = ManuallyDrop::new(self);
    let ptr = vec.ptr;
    let cap = vec.cap;
    let len = vec.len;

    IntoIter { 
      buf: ptr, 
      cap, 
      start: ptr.as_ptr(), 
      end:if cap == 0 {
            ptr.as_ptr()
          }
          else{
            unsafe { ptr.as_ptr().add(len) }
          },
    }
  }
}

// iterating forward
impl<T> Iterator for IntoIter<T>{
  type Item = T; 
  
  fn next(&mut self) -> Option<T>{
    if self.start == self.end {
      None
    }
    else{
      unsafe{
        let res = std::ptr::read(self.start);
        self.start = self.start.offset(1);
        Some(res)
      }
    }
  }

  fn size_hint(&self) -> (usize, Option<usize>){
    let len = (self.end as usize - self.start as usize) / mem::size_of::<T>();
    
    return (len, Some(len));
  }
}

// iterating backwards
impl<T> DoubleEndedIterator for IntoIter<T> {
  fn next_back(&mut self) -> Option<Self::Item> {
      if self.start == self.end{
        None
      }
      else{
        unsafe{
          self.end = self.end.offset(-1); 
          Some(std::ptr::read(self.end))
        }
      }
  }
}

// Intoiter takes ownership of its alloc. -> Drop needed to free it
impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        if self.cap != 0 {
            // drop any remaining elements
            for _ in &mut *self {}
            let layout = Layout::array::<T>(self.cap).unwrap();
            unsafe {
                alloc::dealloc(self.buf.as_ptr() as *mut u8, layout);
            }
        }
    }
}