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
        let mut cur_node = self.root;
        let mut length = 0;
        let mut depth = 0;
        let mut index = 0;
        while True {
            let ptr = cur_node.inner;
            let node = unsafe { Box::from_raw(ptr as *mut CachePadded<(NodeHeader, NodeBodyV<V>)>) };
            let (header, body) = CachePadded::into_inner(*node);
            for i in range(0, header.key.len()) {
                // what if key is not equal? 
                key.next();
                if key == KEY_ENDMARK {
                    // case when cursor points to edge, and value is not yet there
                    //construct cursor; we return edge
                    length += 1;
                    depth += 1;
                }
                length += 1;
                depth += 1;
            }
            //if out of the loop, then if key.next() is endmark we need to return value, else, continue traversing to next node
            length = 0; //resetting variable
            key.next();
            let (i, node_box) = body.lookup(key);
            index = i;
            cur_node = node_box;//moving to the next box
            if key == KEY_ENDMARK && cur_node.deref_mut().unwrap().1.is_right() { //it is leaf 
                //construct cursor; we are in the destination leaf
            }
            // if we are here, then continue treversal 
        }
        node = root
        while True:
            key = key.next()
            if key == key endmark:
                //we are in NodeBoxV, extract value
                value = node.inner 
                break
            for child in children:
                    if child.header.key[0] == key:
                        node = child 
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
