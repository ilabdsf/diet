#![feature(step_trait)]
#![feature(box_syntax)]

use std::iter::Step;
use std::mem;

type Link<T> = Option<Box<Node<T>>>;

#[derive(Default)]
pub struct Diet<T: Ord + Step> {
    root: Link<T>,
}


// `Node` in a `Diet`
struct Node<T: Ord + Step> {
    segment: Segment<T>,
    left: Link<T>,
    right: Link<T>,
}

// The original paper calls it interval,
// but actually it includes both ends, so it is a segment.
#[derive(Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Segment<T> {
    left: T,
    right: T,
}

impl<T: Ord + Step> Node<T> {
    pub fn new(segment: Segment<T>) -> Self {
        Node {
            segment: segment,
            left: None,
            right: None,
        }
    }

    /// Remove all intervals adjacent to `left` and return the leftmost boundary
    /// to extend the root.
    pub fn consume_left_link(link: &mut Link<T>, left: T) -> T {
        let leftptr;
        if let Some(ref mut node) = *link {
            if node.segment.right.add_one() < left {
                // This one is not adjacent, just descend into the right subtree.
                return Node::consume_left_link(&mut node.right, left);
            } else {
                // Adjacent, consume it along with its right subtree.

                // Detach left pointer
                leftptr = mem::replace(&mut node.left, None);
                // Fall through to release `node` which borrows from link
            }
        } else {
            return left;
        }
        mem::replace(link, leftptr).unwrap().segment.left
    }

    /// Similar to consume_right_link
    pub fn consume_right_link(link: &mut Link<T>, right: T) -> T {
        let rightptr;
        if let Some(ref mut node) = *link {
            if node.segment.left.sub_one() > right {
                return Node::consume_right_link(&mut node.left, right);
            } else {
                rightptr = mem::replace(&mut node.right, None);
            }
        } else {
            return right;
        }
        mem::replace(link, rightptr).unwrap().segment.right
    }

    pub fn insert_link(link: &mut Link<T>, segment: Segment<T>) {
        if let Some(ref mut node) = *link {
            node.insert(segment);
        } else {
            *link = Some(box Node::new(segment));
        }
    }

    pub fn insert(&mut self, segment: Segment<T>) {
        if segment.right < self.segment.left {
            if segment.right < self.segment.left.sub_one() {
                // Segments are not adjacent, just insert new segment into the left subtree.
                Node::insert_link(&mut self.left, segment);
            } else {
                // Extend the root and keep removing maximum elements from left subtrees until we
                // find non-adjacent one. Use each removed element to extend the root.
                self.segment.left = Node::consume_left_link(&mut self.left, segment.left);
            }
        } else if segment.left > self.segment.right {
            if segment.left > self.segment.right.add_one() {
                // Segments are not adjacent, just insert new segment into the right subtree.
                Node::insert_link(&mut self.right, segment);
            } else {
                // Adjacent
                self.segment.right = Node::consume_right_link(&mut self.right, segment.right);
            }
        } else {
            if segment.left < self.segment.left {
                self.segment.left = Node::consume_left_link(&mut self.left, segment.left);
            }
            if segment.right > self.segment.right {
                self.segment.right = Node::consume_right_link(&mut self.right, segment.right);
            }
        }
    }

    pub fn contains(&self, value: &T) -> bool {
        if value < &self.segment.left {
            if let Some(ref left) = self.left {
                left.contains(value)
            } else {
                false
            }
        } else if value > &self.segment.right {
            if let Some(ref right) = self.right {
                right.contains(value)
            } else {
                false
            }
        } else {
            true
        }
    }
}

impl<T: Ord + Step> Diet<T> {
    pub fn new() -> Self {
        Diet { root: None }
    }

    /// Insert `segment` into `Diet`
    ///
    /// # Examples
    ///
    /// ```
    /// use diet::{Diet, Segment};
    ///
    /// let mut diet: Diet<i32> = Diet::new();
    /// diet.insert(Segment::new(5, 9));
    /// assert!(diet.contains(&5));
    /// assert!(!diet.contains(&4));
    /// diet.insert(Segment::new(-5, 7));
    /// assert!(diet.contains(&4));
    /// assert!(diet.contains(&9));
    /// ```
    pub fn insert(&mut self, segment: Segment<T>) {
        if let Some(ref mut root) = self.root {
            root.insert(segment);
        } else {
            self.root = Some(box Node::new(segment));
        }
    }

    /// Returns `true` if `Diet` is empty
    ///
    /// # Examples
    ///
    /// ```
    /// use diet::Diet;
    ///
    /// let diet: Diet<i32> = Diet::new();
    /// assert!(diet.is_empty())
    /// ```
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    /// Clears the diet, removing all values.
    #[inline]
    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn contains(&self, value: &T) -> bool {
        if let Some(ref node) = self.root {
            node.contains(value)
        } else {
            false
        }
    }
}

impl<T: Ord> Segment<T> {
    pub fn new(left: T, right: T) -> Self {
        assert!(left <= right);
        Segment {
            left: left,
            right: right,
        }
    }

    /// Returns `true` if the segment contains a value.
    ///
    /// # Examples
    ///
    /// ```
    /// use diet::Segment;
    ///
    /// let segment = Segment::new(1, 5);
    /// assert!(segment.contains(&5));
    /// assert!(!segment.contains(&6));
    /// ```
    pub fn contains(&self, value: &T) -> bool {
        &self.left <= value && value <= &self.right
    }

    pub fn left(&self) -> &T {
        &self.left
    }

    pub fn right(&self) -> &T {
        &self.right
    }
}

pub struct DietIterator<T: Ord + Step> {
    queue: Vec<Box<Node<T>>>,
}

impl<T: Ord + Step> IntoIterator for Diet<T> {
    type Item = Segment<T>;
    type IntoIter = DietIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        let mut iter = DietIterator { queue: Vec::new() };
        iter.descend(self.root);
        iter
    }
}

impl<T: Ord + Step> DietIterator<T> {
    fn descend(&mut self, mut current: Link<T>) {
        loop {
            if current.is_none() {
                break;
            }
            let mut node = current.take().unwrap();
            current = node.left.take();
            self.queue.push(node);
        }
    }
}

impl<T: Ord + Step> Iterator for DietIterator<T> {
    type Item = Segment<T>;

    fn next(&mut self) -> Option<Segment<T>> {
        if let Some(mut result) = self.queue.pop() {
            if let Some(right) = result.right.take() {
                self.descend(Some(right));
            }
            Some(result.segment)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_consuming_iterator() {
        let mut diet = Diet::new();
        diet.insert(Segment::new(5, 15));
        diet.insert(Segment::new(20, 40));
        diet.insert(Segment::new(100, 200));
        diet.insert(Segment::new(10, 25));
        let v: Vec<Segment<i32>> = diet.into_iter().collect();
        assert_eq!(vec![Segment::new(5, 40), Segment::new(100, 200)], v);
    }
}
