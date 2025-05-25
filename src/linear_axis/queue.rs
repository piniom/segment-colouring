use std::{fmt::Debug, iter::zip};

#[derive(Debug, Clone)]
pub struct Queue<T> {
    nodes: Vec<Node<T>>,
    first: usize,
    last: usize,
    free: usize,
    len: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Node<T> {
    next: usize,
    prev: usize,
    val: T,
}

impl<T: Default> Node<T> {
    pub fn default_value(next: usize, prev: usize) -> Self {
        Self::new(next, prev, T::default())
    }
    pub fn new(next: usize, prev: usize, val: T) -> Self {
        Self { next, prev, val }
    }
}

impl<T: Default + Debug + Clone> Queue<T> {
    pub fn new() -> Self {
        Self::with_capacity(30)
    }
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            nodes: (0..capacity)
                .map(|i| Node::default_value((i + 1) % capacity, 0))
                .collect(),
            first: 0,
            last: 0,
            len: 0,
            free: 0,
        }
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn push_back(&mut self, val: T) {
        if self.len + 1 == self.nodes.len() {
            dbg!(&self);
            panic!("overflow")
        }
        if self.len == 0 {
            self.first = self.free;
        } else {
            self.nodes[self.last].next = self.free;
        }
        self.nodes[self.free].val = val;
        self.nodes[self.free].prev = self.last;
        self.last = self.free;
        self.free = self.nodes[self.free].next;
        self.len += 1;
    }
    pub fn push_front(&mut self, val: T) {
        if self.len + 1 == self.nodes.len() {
            panic!("overflow")
        }
        if self.len == 0 {
            self.last = self.free;
        } else {
            self.nodes[self.first].prev = self.free;
        }
        let next_free = self.nodes[self.free].next;
        self.nodes[self.free].val = val;
        self.nodes[self.free].next = self.first;
        self.first = self.free;
        self.free = next_free;
        self.len += 1;
    }
    pub fn pop_front(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        let mut val = T::default();
        std::mem::swap(&mut val, &mut self.nodes[self.first].val);
        let next_free = self.nodes[self.first].next;
        self.nodes[self.first].next = self.free;
        self.free = self.first;
        self.first = next_free;
        Some(val)
    }
    pub fn pop_back(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        let mut val = T::default();
        std::mem::swap(&mut val, &mut self.nodes[self.last].val);
        self.nodes[self.last].next = self.free;
        self.free = self.last;
        self.last = self.nodes[self.last].prev;
        Some(val)
    }
    pub fn remove_at_index(&mut self, index: usize) -> Option<T> {
        if index >= self.len {
            return None
        }
        if index == 0 {
            return self.pop_front();
        }
        if index == self.len - 1 {
            return self.pop_back();
        }
        let found = self.get_ith(index);
        let Node { next, prev, .. } = self.nodes[found];
        self.nodes[prev].next = next;
        self.nodes[next].prev = prev;
        self.nodes[found].next = self.free;
        self.free = found;
        self.len -= 1;
        Some(self.nodes[found].val.clone())
    }
    pub fn insert_at_index(&mut self, index: usize, val: T) -> Option<()> {
        if index == 0 {
            self.push_front(val);
            return Some(());
        }
        if index == self.len {
            self.push_back(val);
            return Some(());
        }
        if index > self.len {
            return None
        }
        let found = self.get_ith(index);
        let next_free = self.nodes[self.free].next;
        let prv = self.nodes[found].prev;
        self.nodes[self.free] = Node {
            next: found,
            prev: prv,
            val
        };
        self.nodes[found].prev = self.free;
        self.nodes[prv].next = self.free;
        self.free = next_free;
        self.len += 1;
        Some(())
    }
    pub fn extend(&mut self, iter: impl IntoIterator<Item = T>) {
        for i in iter {
            self.push_back(i);
        }
    }
    pub fn iter<'a>(&'a self) -> QueueIterator<'a, T> {
        self.into_iter()
    }
    pub fn get<'a>(&'a self, index: usize) -> Option<&'a T> {
        self.iter().skip(index).next()
    }
    fn get_ith(&self, i: usize) -> usize {
        if i > self.len / 2 {
            return self.get_ith_back(self.len - i - 1)
        }
        let mut result = self.first;
        for _ in 0..i {
            result = self.nodes[result].next;
        }
        result
    }
    fn get_ith_back(&self, i: usize) -> usize {
        let mut result = self.last;
        for _ in 0..i {
            result = self.nodes[result].prev;
        }
        result
    }
}

impl<T: PartialEq> PartialEq for Queue<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.len != other.len {
            return false;
        }
        zip(self.into_iter(), other.into_iter()).all(|(a, b)| a == b)
    }
}

impl<'a, T> IntoIterator for &'a Queue<T> {
    type Item = &'a T;

    type IntoIter = QueueIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        QueueIterator {
            queue: self,
            cur_front: self.first,
            cur_back: self.last,
            done: 0,
        }
    }
}

pub struct QueueIterator<'a, T> {
    queue: &'a Queue<T>,
    cur_front: usize,
    cur_back: usize,
    done: usize,
}

impl<'a, T> Iterator for QueueIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done == self.queue.len {
            return None;
        }
        let node = &self.queue.nodes[self.cur_front];
        self.cur_front = node.next;
        self.done += 1;
        Some(&node.val)
    }
}

impl<'a, T> DoubleEndedIterator for QueueIterator<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.done == self.queue.len {
            return None;
        }
        let node = &self.queue.nodes[self.cur_back];
        self.cur_back = node.prev;
        self.done += 1;
        Some(&node.val)
    }
}

impl<T: Default + Debug + Clone> FromIterator<T> for Queue<T> {
    fn from_iter<R: IntoIterator<Item = T>>(iter: R) -> Queue<T> {
        let mut q = Queue::new();
        q.extend(iter);
        q
    }
}

impl<R, T> From<R> for Queue<T>
where
    T: Default + Debug + Clone,
    R: IntoIterator<Item = T>,
{
    fn from(value: R) -> Self {
        Queue::from_iter(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let q: Queue<i32> = Queue::new();
        assert_eq!(q.len(), 0);
        assert!(q.is_empty());
    }

    #[test]
    fn test_with_capacity() {
        let q: Queue<i32> = Queue::with_capacity(10);
        assert_eq!(q.nodes.len(), 10);
        assert_eq!(q.len(), 0);
        assert!(q.is_empty());
    }

    #[test]
    fn test_push_back_empty() {
        let mut q = Queue::new();
        q.push_back(1);
        assert_eq!(q.len(), 1);
        assert!(!q.is_empty());
        assert_eq!(q.first, 0);
        assert_eq!(q.last, 0);
        assert_eq!(q.get(0), Some(&1));
    }

    #[test]
    fn test_push_back_non_empty() {
        let mut q = Queue::with_capacity(5);
        q.push_back(1);
        q.push_back(2);
        assert_eq!(q.len(), 2);
        assert_eq!(q.first, 0);
        assert_eq!(q.last, 1);
        dbg!(&q);
        assert_eq!(q.get(1), Some(&2));
        assert_eq!(q.get(0), Some(&1));
    }

    #[test]
    #[should_panic]
    fn test_push_back_overflow() {
        let mut q = Queue::with_capacity(1);
        q.push_back(1);
        q.push_back(2);
    }

    #[test]
    fn test_push_front_empty() {
        let mut q = Queue::new();
        q.push_front(1);
        assert_eq!(q.len(), 1);
        assert!(!q.is_empty());
        assert_eq!(q.first, 0);
        assert_eq!(q.last, 0);
        assert_eq!(q.get(0), Some(&1));
    }

    #[test]
    fn test_push_front_non_empty() {
        let mut q = Queue::with_capacity(5);
        q.push_front(2);
        q.push_front(1);
        assert_eq!(q.len(), 2);
        assert_eq!(q.first, 1);
        assert_eq!(q.last, 0);
        assert_eq!(q.get(0), Some(&1));
        assert_eq!(q.get(1), Some(&2));
    }

    #[test]
    #[should_panic]
    fn test_push_front_overflow() {
        let mut q = Queue::with_capacity(1);
        q.push_front(1);
        q.push_front(2);
    }

    #[test]
    fn test_pop_front_empty() {
        let mut q: Queue<i32> = Queue::new();
        assert_eq!(q.pop_front(), None);
        assert_eq!(q.len(), 0);
        assert!(q.is_empty());
    }

    #[test]
    fn test_pop_front_single() {
        let mut q = Queue::new();
        q.push_back(1);
        assert_eq!(q.pop_front(), Some(1));
        assert_eq!(q.len(), 0);
        assert!(q.is_empty());
    }

    #[test]
    fn test_pop_front_multiple() {
        let mut q = Queue::new();
        q.push_back(1);
        q.push_back(2);
        assert_eq!(q.pop_front(), Some(1));
        assert_eq!(q.len(), 1);
        assert_eq!(q.pop_front(), Some(2));
        assert_eq!(q.len(), 0);
        assert!(q.is_empty());
    }

    #[test]
    fn test_pop_back_empty() {
        let mut q: Queue<i32> = Queue::new();
        assert_eq!(q.pop_back(), None);
        assert_eq!(q.len(), 0);
        assert!(q.is_empty());
    }

    #[test]
    fn test_pop_back_single() {
        let mut q = Queue::new();
        q.push_back(1);
        assert_eq!(q.pop_back(), Some(1));
        assert_eq!(q.len(), 0);
        assert!(q.is_empty());
    }

    #[test]
    fn test_pop_back_multiple() {
        let mut q = Queue::new();
        q.push_back(1);
        q.push_back(2);
        assert_eq!(q.pop_back(), Some(2));
        assert_eq!(q.len(), 1);
        assert_eq!(q.pop_back(), Some(1));
        assert_eq!(q.len(), 0);
        assert!(q.is_empty());
    }

    #[test]
    fn test_extend_empty() {
        let mut q: Queue<i32> = Queue::new();
        q.extend(vec![]);
        assert_eq!(q.len(), 0);
        assert!(q.is_empty());
    }

    #[test]
    fn test_extend_single() {
        let mut q: Queue<i32> = Queue::new();
        q.extend(vec![1]);
        assert_eq!(q.len(), 1);
        assert_eq!(q.get(0), Some(&1));
    }

    #[test]
    fn test_extend_multiple() {
        let mut q: Queue<i32> = Queue::new();
        q.extend(vec![1, 2, 3]);
        assert_eq!(q.len(), 3);
        assert_eq!(q.get(0), Some(&1));
        assert_eq!(q.get(1), Some(&2));
        assert_eq!(q.get(2), Some(&3));
    }

    #[test]
    fn test_iter_empty() {
        let q: Queue<i32> = Queue::new();
        let mut iter = q.iter();
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_iter_single() {
        let mut q = Queue::new();
        q.push_back(1);
        let mut iter = q.iter();
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_iter_multiple() {
        let mut q = Queue::new();
        q.push_back(1);
        q.push_back(2);
        let mut iter = q.iter();
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_into_iter_empty() {
        let q: Queue<i32> = Queue::new();
        let mut iter = q.into_iter();
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_into_iter_single() {
        let mut q = Queue::new();
        q.push_back(1);
        let mut iter = q.into_iter();
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_into_iter_multiple() {
        let mut q = Queue::new();
        q.push_back(1);
        q.push_back(2);
        let mut iter = q.into_iter();
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_get_empty() {
        let q: Queue<i32> = Queue::new();
        assert_eq!(q.get(0), None);
    }

    #[test]
    fn test_get_single() {
        let mut q = Queue::new();
        q.push_back(1);
        assert_eq!(q.get(0), Some(&1));
        assert_eq!(q.get(1), None);
    }

    #[test]
    fn test_get_multiple() {
        let mut q = Queue::new();
        q.push_back(1);
        q.push_back(2);
        q.push_back(3);
        assert_eq!(q.get(0), Some(&1));
        assert_eq!(q.get(1), Some(&2));
        assert_eq!(q.get(2), Some(&3));
        assert_eq!(q.get(3), None);
    }

    #[test]
    fn test_partial_eq_empty() {
        let q1: Queue<i32> = Queue::new();
        let q2: Queue<i32> = Queue::new();
        assert_eq!(q1, q2);
    }

    #[test]
    fn test_partial_eq_single_equal() {
        let mut q1 = Queue::new();
        q1.push_back(1);
        let mut q2 = Queue::new();
        q2.push_back(1);
        assert_eq!(q1, q2);
    }

    #[test]
    fn test_partial_eq_single_not_equal() {
        let mut q1 = Queue::new();
        q1.push_back(1);
        let mut q2 = Queue::new();
        q2.push_back(2);
        assert_ne!(q1, q2);
    }

    #[test]
    fn test_partial_eq_multiple_equal() {
        let mut q1 = Queue::new();
        q1.extend(vec![1, 2, 3]);
        let mut q2 = Queue::new();
        q2.extend(vec![1, 2, 3]);
        assert_eq!(q1, q2);
    }

    #[test]
    fn test_partial_eq_multiple_not_equal_length() {
        let mut q1 = Queue::new();
        q1.extend(vec![1, 2]);
        let mut q2 = Queue::new();
        q2.extend(vec![1, 2, 3]);
        assert_ne!(q1, q2);
    }

    #[test]
    fn test_partial_eq_multiple_not_equal_elements() {
        let mut q1 = Queue::new();
        q1.extend(vec![1, 2, 3]);
        let mut q2 = Queue::new();
        q2.extend(vec![1, 4, 3]);
        assert_ne!(q1, q2);
    }

    #[test]
    fn test_from_iterator_empty() {
        let q: Queue<i32> = vec![].into_iter().collect();
        assert_eq!(q.len(), 0);
        assert!(q.is_empty());
    }

    #[test]
    fn test_from_iterator_single() {
        let q: Queue<i32> = vec![1].into_iter().collect();
        assert_eq!(q.len(), 1);
        assert_eq!(q.get(0), Some(&1));
    }

    #[test]
    fn test_from_iterator_multiple() {
        let q: Queue<i32> = vec![1, 2, 3].into_iter().collect();
        assert_eq!(q.len(), 3);
        assert_eq!(q.get(0), Some(&1));
        assert_eq!(q.get(1), Some(&2));
        assert_eq!(q.get(2), Some(&3));
    }

    #[test]
    fn test_from_vec_empty() {
        let q: Queue<i32> = Vec::new().into();
        assert_eq!(q.len(), 0);
        assert!(q.is_empty());
    }

    #[test]
    fn test_from_vec_single() {
        let q: Queue<i32> = vec![1].into();
        assert_eq!(q.len(), 1);
        assert_eq!(q.get(0), Some(&1));
    }

    #[test]
    fn test_from_vec_multiple() {
        let q: Queue<i32> = vec![1, 2, 3].into();
        assert_eq!(q.len(), 3);
        assert_eq!(q.get(0), Some(&1));
        assert_eq!(q.get(1), Some(&2));
        assert_eq!(q.get(2), Some(&3));
    }

    #[test]
    fn test_double_ended_iterator_empty() {
        let q: Queue<i32> = Queue::new();
        let mut iter = q.iter();
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn test_double_ended_iterator_single() {
        let mut q = Queue::new();
        q.push_back(1);
        let mut iter = q.iter();
        assert_eq!(iter.next_back(), Some(&1));
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn test_double_ended_iterator_multiple() {
        let mut q = Queue::new();
        q.push_back(1);
        q.push_back(2);
        q.push_back(3);
        let mut iter = q.iter();
        assert_eq!(iter.next_back(), Some(&3));
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next_back(), Some(&2));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn test_pop_front_and_push_back_interleaved() {
        let mut q = Queue::with_capacity(5);
        q.push_back(1);
        q.push_back(2);
        assert_eq!(q.pop_front(), Some(1));
        q.push_back(3);
        assert_eq!(q.pop_front(), Some(2));
        q.push_back(4);
        assert_eq!(q.pop_front(), Some(3));
        assert_eq!(q.pop_front(), Some(4));
        assert!(q.is_empty());
    }

    #[test]
    fn test_pop_back_and_push_front_interleaved() {
        let mut q = Queue::with_capacity(5);
        q.push_front(1);
        q.push_front(2);
        assert_eq!(q.pop_back(), Some(1));
        q.push_front(3);
        assert_eq!(q.pop_back(), Some(2));
        q.push_front(4);
        assert_eq!(q.pop_back(), Some(3));
        assert_eq!(q.pop_back(), Some(4));
        assert!(q.is_empty());
    }
}