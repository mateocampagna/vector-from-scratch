mod vector;

use vector::Vector;

fn main() {
    // ---------------- push ----------------

    let mut v = Vector::new();

    v.push(10);
    v.push(20);
    v.push(30);

    assert_eq!(v.len(), 3); // esto no compila porque len es privado

    println!("After push: {:?}", &*v);

    // ---------------- pop ----------------

    assert_eq!(v.pop(), Some(30));
    assert_eq!(v.pop(), Some(20));
    assert_eq!(v.pop(), Some(10));
    assert_eq!(v.pop(), None);

    // ---------------- insert ----------------

    v.push(10);
    v.push(30);

    v.insert(1, 20);

    println!("After insert: {:?}", &*v);

    assert_eq!(v[0], 10);
    assert_eq!(v[1], 20);
    assert_eq!(v[2], 30);

    // ---------------- remove ----------------

    let removed = v.remove(1);

    assert_eq!(removed, 20);
    assert_eq!(v[0], 10);
    assert_eq!(v[1], 30);

    // ---------------- DerefMut ----------------

    v[0] = 100;

    assert_eq!(v[0], 100);

    // ---------------- IntoIterator ----------------

    let mut v2 = Vector::new();

    v2.push(1);
    v2.push(2);
    v2.push(3);

    let values: Vec<_> = v2.into_iter().collect();

    assert_eq!(values, vec![1, 2, 3]);

    println!("All tests passed!");
}