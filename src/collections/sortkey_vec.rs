use std::iter;



#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SortkeyVec<T> {
    vec: Vec<T>,
    keys: Vec<i32>
}

impl<T> SortkeyVec<T> {
    pub fn new() -> Self {
        Self {
            vec: Vec::new(),
            keys: Vec::new()
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            vec: Vec::with_capacity(capacity),
            keys: Vec::with_capacity(capacity)
        }
    }

    #[inline(always)]
    fn get_potential_index(&self, index: i32) -> usize {
        let mut low = 0;
        let mut high = self.keys.len();
        let mut mid;

        while low < high {
            mid = (low + high) / 2;
            if *unsafe{self.keys.get_unchecked(mid)} < index {
                low = mid + 1;
            } else {
                high = mid;
            }
        }
        low
    }

    pub fn get(&self, index: i32) -> Option<&T> {
        let idx = self.keys.binary_search(&index).ok()?;
        self.vec.get(idx)
    }

    pub fn get_mut(&mut self, index: i32) -> Option<&mut T> {
        let idx = self.keys.binary_search(&index).ok()?;
        self.vec.get_mut(idx) 
    }

    pub fn insert(&mut self, index: i32, value: T) -> Option<T> {
        if self.keys.last().map_or(true, |&last| index > last) {
            self.keys.push(index);
            self.vec.push(value);
            return None;
        }
        match self.keys.binary_search(&index) {
            Ok(i) => Some(std::mem::replace(&mut self.vec[i], value)),
            Err(i) => {
                self.keys.insert(i, index);
                self.vec.insert(i, value);
                None
            }
        }
    }

    pub fn remove(&mut self, index: i32) -> Option<T> {
        let idx = self.keys.binary_search(&index).ok()?;
        self.keys.remove(idx);
        Some(self.vec.remove(idx))
    }

    pub fn find_index(&self, value: &T) -> Option<usize> where T: PartialEq {
        for (i, v) in self.vec.iter().enumerate() {
            if v == value {
                return Some(i);
            }
        }
        None
    }

    pub fn iter_keys(&self) -> std::slice::Iter<'_, i32> {
        self.keys.iter()
    }
    pub fn iter_values(&self) -> std::slice::Iter<'_, T> {
        self.vec.iter()
    }
    pub fn iter_mut_values(&mut self) -> std::slice::IterMut<'_, T> {
        self.vec.iter_mut()
    }
}


impl<T> iter::IntoIterator for SortkeyVec<T> {
  type Item = (i32, T);
  type IntoIter = iter::Zip<std::vec::IntoIter<i32>, std::vec::IntoIter<T>>;
  fn into_iter(self) -> Self::IntoIter {
    self.keys.into_iter().zip(self.vec.into_iter())
  }
}