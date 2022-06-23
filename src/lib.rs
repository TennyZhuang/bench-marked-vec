#![feature(test)]
#![feature(core_intrinsics)]
#![feature(generators, generator_trait)]

use std::{pin::Pin, ops::{GeneratorState, Generator}};


extern crate test;

pub struct MarkedVec {
    v: Vec<i32>,
    vis: Vec<bool>,
}

impl MarkedVec {
    pub fn iter(&self) -> MarkedVecIter<'_> {
        MarkedVecIter::<'_> {
            v: &self.v, vis: &self.vis, pos: 0,
        }
    }
}

pub struct MarkedVecIter<'a> {
    v: &'a [i32],
    vis: &'a [bool],
    pos: usize,
}

impl<'a> Iterator for MarkedVecIter<'a> {
    type Item = i32;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            while self.pos < self.v.len() {
                if *self.vis.get_unchecked(self.pos) {
                    let item = *self.v.get_unchecked(self.pos);
                    self.pos += 1;
                    return Some(item);
                }
                self.pos += 1;
            }
            None
        }
    }
}

#[no_mangle]
pub fn g(v: i32) {
    let _ = v * v * v;
}

pub fn iter1(mv: &MarkedVec) {
    let n = mv.v.len();
    for i in 0..n {
        unsafe {
            if *mv.vis.get_unchecked(i) {
                g(*mv.v.get_unchecked(i));
            }
        }
    }
}

pub fn iter2(mv: &MarkedVec) {
    for x in mv.iter() {
        g(x);
    }
}

pub fn iter3(mv: &MarkedVec) {
    let mut generatar = || {
        let n = mv.v.len();
        for i in 0..n {
            unsafe {
                if *mv.vis.get_unchecked(i) {
                    yield *mv.v.get_unchecked(i);
                }
            }
        }
    };

    while let GeneratorState::Yielded(x) = Pin::new(&mut generatar).resume(()) {
        g(x);
    }
}

#[inline(never)]
fn init(mv: &mut MarkedVec, n: usize, x: i32, y: i32) {
    mv.v.clear();
    for i in 0..n {
        mv.v.push(x + i as i32);
    }
    mv.vis.clear();
    for i in 0..n {
        mv.vis.push(i % y as usize == 0);
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[test]
    fn it_works() {
    }

    #[bench]
    fn bench1(b: &mut Bencher) {
        let n = test::black_box(100000);
        let mut mv = MarkedVec {
            v: Vec::with_capacity(n),
            vis: Vec::with_capacity(n),
        };
        b.iter(|| {
            init(&mut mv, n, 3, 2);
            iter1(&mv);
        });
    }

    #[bench]
    fn bench2(b: &mut Bencher) {
        let n = test::black_box(100000);
        let mut mv = MarkedVec {
            v: Vec::with_capacity(n),
            vis: Vec::with_capacity(n),
        };
        b.iter(|| {
            init(&mut mv, n, 3, 2);
            iter2(&mv);
        });
    }

    #[bench]
    fn bench3(b: &mut Bencher) {
        let n = test::black_box(100000);
        let mut mv = MarkedVec {
            v: Vec::with_capacity(n),
            vis: Vec::with_capacity(n),
        };
        b.iter(|| {
            init(&mut mv, n, 3, 2);
            iter3(&mv);
        });
    }
}
