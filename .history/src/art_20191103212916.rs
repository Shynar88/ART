use core::iter::Peekable;
use core::ptr;

use crate::map::*;
use crate::node::*;

/// Adaptive radix tree.
///
/// IMPORTANT: you should not change this type.
#[derive(Debug)]
pub struct Art<V> {
    root: NodeBox<V>,
}

#[derive(Debug)]
struct Cursor<'a, V> {
    depth: usize,
    parent: Option<&'a mut NodeBox<V>>,
    child: &'a mut NodeBox<V>,
    index: u8,
    length: u8,
}

/// Entry API for Art.
///
/// See https://doc.rust-lang.org/std/collections/hash_map/enum.Entry.html for more details of the
/// entry API.
#[derive(Debug)]
pub struct Entry<'a, V, I: Iterator<Item = u8> + DoubleEndedIterator> {
    cursor: Cursor<'a, V>,
    key: Peekable<I>,
}

impl<'a, V, I: 'a + Iterator<Item = u8> + DoubleEndedIterator> Entry<'a, V, I> {
    /// Inserts the generated value if the entry is vacant.
    ///
    /// Returns `Ok(v)` if inserted, where `v` is a mutable reference to the inserted value;
    /// `Err((v, f))` if not inserted, where `v` is a mutable reference to the existing value and
    /// `f` is the given value generator.
    #[inline]
    pub fn or_insert_with<F>(mut self, f: F) -> Result<&'a mut V, (&'a mut V, F)> //Entry, 
    where
        F: FnOnce() -> V,
    {
        unimplemented!() //call insert on remaining key (new_path?)
    }

    /// Inserts the given value if the entry is vacant.
    ///
    /// Returns `Ok(v)` if inserted, where `v` is a mutable reference to the inserted value;
    /// `Err((v, f))` if not inserted, where `v` is a mutable reference to the existing value and
    /// `f` is the given value generator.
    pub fn or_insert(self, default: V) -> Result<&'a mut V, (&'a mut V, V)> {
        self.or_insert_with(|| default).map_err(|(v, f)| (v, f()))
    }

    /// Provides in-place mutable access to an occupied entry before any potential inserts into the
    /// map.
    pub fn and_modify<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut V),
    {
        if let Some(v) = self.lookup() {
            f(v);
        }

        self
    }

    /// Deletes the value at the entry.
    ///
    /// Returns `Ok(v)` if the entry contains a value, `v`; `Err(())` if the entry does not contain
    /// a value.
    pub fn delete(mut self) -> Result<V, ()> {
        unimplemented!()
    }

    /// Lookups the entry's value.
    pub fn lookup(&mut self) -> Option<&mut V> {
        if self.key.peek().is_some() {
            return None;
        }

        let (header, body) = self.cursor.child.deref_mut().unwrap(); //returns NodeBox's inner: (header, b)
        assert_eq!(self.cursor.length, header.length());
        body.right() //is a reference to the leaf node's value
    }
}

impl<V> Default for Art<V> {
    fn default() -> Self {
        Self {
            root: NodeBox::newi(NodeHeader::default(), vec![], 256),
        }
    }
}

impl<V> Art<V> {
    /// Encodes a given string into an array of `u8`. Appending a sentinel value (0xff) to make sure
    /// a string is not a prefix of another.
    fn encode_key<'a>(key: &'a str) -> impl 'a + Iterator<Item = u8> + DoubleEndedIterator {
        key.bytes().chain(vec![KEY_ENDMARK].into_iter())
    }

    /// Creates an adaptive radix tree.
    pub fn new() -> Self {
        Self::default()
    }

    fn cursor<'a, I>(&'a mut self, key: &mut Peekable<I>) -> Cursor<'a, V> //(ART, whole key) -> Cursor
    where 
        I: 'a + Iterator<Item = u8>,
    {
        let mut cur_node = &mut self.root;
        let mut parent = None;
        let mut length = 0;
        let mut depth = 0;
        let mut index = 0;
        loop {
            let (header, b) = cur_node.deref_mut().unwrap();
            let body = b.left().unwrap();
            for i in 0..header.key().len() {
                match key.peek(){
                    None => {break},
                    Some(v) => {
                        if *v == header.key()[i] {
                            key.next();
                            length += 1;
                            depth += 1;
                        } else { // case when cursor points to edge, and value is not yet there. construct cursor; we return edge
                            break;
                        }
                    },
                } 
            }
            if key.peek() == None || header.key.len() < length {
                break;
            }

            //if out of the loop, then if key.next() is endmark we need to return value, else, continue traversing to next node
            length = 0; //resetting variable
            let result = body.lookup_mut(key.peek()); //might fail if key doesn't exist
            let node_box: &mut NodeBox<V>; 
            match result {
                Some(v) => {
                    (index, node_box) = result;
                },
                None => {
                    break
                    }, 
            }
            parent = cur_node;
            cur_node = node_box; //moving to the next box
        }
        //return Cursor here
        Cursor {
                depth: depth,
                parent: parent,
                child: cur_node,
                index: index,
                length: length
            }
    }

    /// Creates an entry.
    pub fn entry<'a, I>(&'a mut self, key: I) -> Entry<'a, V, I>
    where
        I: 'a + Iterator<Item = u8> + DoubleEndedIterator,
    {
        let mut key = key.peekable();
        let cursor = self.cursor(&mut key);
        Entry { cursor, key }
    }
}

impl<V> SequentialMap<V> for Art<V> {
    fn insert<'a>(&'a mut self, key: &'a str, value: V) -> Result<&'a mut V, (&'a mut V, V)> {
        let key = Self::encode_key(key);
        self.entry(key).or_insert(value)
    }

    fn delete(&mut self, key: &str) -> Result<V, ()> {
        let key = Self::encode_key(key);
        self.entry(key).delete()
    }

    fn lookup<'a>(&'a self, key: &'a str) -> Option<&'a V> {
        let key = Self::encode_key(key);
        unsafe {
            (*(self as *const _ as *mut Self))
                .entry(key)
                .lookup()
                .map(|v| &*(v as *const V))
        }
    }
}
