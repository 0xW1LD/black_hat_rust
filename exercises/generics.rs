use std::fmt::{Debug, Display};
/*
Generics and Traits in rust

Notes:
- Display isn't implemented but serves to add as a template for multiple trait bounds
- the Key struct shows a shorthand implentation of Traitbounds while the BorrowedKey struct shows usage of the where idiom, these are functionally identical.
- Doesn't go over Higher-Rank Trait Bounds
*/

// Const Generic, Generic Type
struct Key<const N: usize, T>{
    data: [T; N],
}

// Combined Trait Bounds, Restricted to Keys with length: 8
impl<T: Debug + Display> Key<8, T> {
    fn length_8(&self) {
        println!("Key is 8 bytes long. \n {:#?}", self.data);
    }
}

// Trait Method
impl<const N: usize,T: Debug + Display> Key<N,T> {
    fn as_borrowed(&self) -> BorrowedKey<'_,T>{
        BorrowedKey { data: &self.data[..self.data.len()/2] } // if 1 will return empty array and will always round down if odd
    }
}

// Lifetime annotations and references in types
struct BorrowedKey<'a, T> {
    data: &'a [T], 
}

// where clause Trait Bounds
impl<'a, T> BorrowedKey<'a, T> 
where
    T: Debug + Display
{
    fn print_borrow(&self){
        println!("Content is borrowing {} bytes: \n {:#?}", self.data.len(), self.data)
    }
}

fn main() {
    let first: Key<8, char>= Key {data: ['p','a','s','s','w','o','r','d']};
    first.length_8();

    let first_borrow: BorrowedKey<'_, char>= BorrowedKey { data: &first.data[0..1] };
    first_borrow.print_borrow();

    let second_borrow: BorrowedKey<'_, char> = first.as_borrowed();
    second_borrow.print_borrow();
}