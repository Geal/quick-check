/* vim: sts=4 sw=4 et
 */

/*!

qc.rs -- QuickCheck for Rust

Use `quick_check` to check that a specified property holds
for values of `trait Arbitrary + Shrink`.

Example::

    extern mod qc;

    fn main() {
        qc::quick_check("sort", qc::config.verbose(true).trials(500),
            |mut v: ~[u8]| { sort(&mut v); is_sorted(v) });
    }

Issues:

* Clean up Lazy and Shrink, implement Arbitrary and Shrink further

---

Copyright License for qc.rs is identical with the Rust project:

'''
Licensed under the Apache License, Version 2.0
<LICENSE-APACHE or
http://www.apache.org/licenses/LICENSE-2.0> or the MIT
license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
at your option. All files in the project carrying such
notice may not be copied, modified, or distributed except
according to those terms.
'''

*/

use lazy::Lazy;
use shrink::Shrink;
use arbitrary::{Arbitrary, arbitrary, SmallN, Unicode};


mod lazy;
mod shrink;
mod arbitrary;


pub struct QConfig {
    trials: uint,
    size: uint,
    verbose: bool,
    grow: bool,
}

/** Default config value */
pub static config: QConfig = QConfig{ trials: 50, size: 8, verbose: false, grow: true };

impl QConfig {
    /// Set size factor (default 8)
    pub fn size(self, x: uint) -> QConfig {
        QConfig{size: x, ..self}
    }
    /// Set n trials (default 50)
    pub fn trials(self, x: uint) -> QConfig {
        QConfig{trials: x, ..self}
    }
    /// Set if size factor should gradually increase (default true)
    pub fn grow(self, x: bool) -> QConfig {
        QConfig{grow: x, ..self}
    }
    /// Set verbose (default false)
    pub fn verbose(self, x: bool) -> QConfig {
        QConfig{verbose: x, ..self}
    }
}

/**
 
 Repeatedly test `property` with values of type `A` chosen using `Arbitrary`.

 If `property` holds true for all tested values, the quick_check test passes.

 If a counterexample is found, quick_check will use `quick_shrink` to try to
 find a minimal counterexample to `property`.

 quick_check calls `fail!()` with an error message indicating `name` and the
 repr of the counterexample.
 
 Examples:
 
 `quick_check!(|x: Type| property(x));`

 `quick_check("name", config, |x: Type| property(x));`

 `quick_check("str", config.trials(100), |s: ~str| s.is_ascii());`
 
 NOTE: `A` must implement `Clone`.
 */
pub fn quick_check<A: Owned + Clone + Shrink + Arbitrary>(name: &str, cfg: QConfig, prop: &fn(A) -> bool) {
    for std::uint::range(0, cfg.trials) |i| {
        let value = arbitrary::<A>(cfg.size + if cfg.grow { i / 8 } else { 0 });
        if cfg.verbose {
            //println(fmt!("qc %s:  %u. trying value '%?'", name, 1+i, &value));
        }
        let v_copy = value.clone();
        if !prop(value) {
            if cfg.verbose {
                println(fmt!("qc %s: first falsification with value '%?'", name, &v_copy));
            }
            let shrink = quick_shrink(cfg, v_copy, prop);
            fail!(fmt!("qc %s: falsified (%u trials) with value '%?'", name, 1+i, shrink));
        }
    }
    if cfg.verbose {
        println(fmt!("qc %s: passed'", name));
    }
}

pub fn quick_shrink<A: Owned + Clone + Shrink + Arbitrary>(cfg: QConfig, value: A, prop: &fn(A) -> bool) -> A {
    //assert!(!prop(value.clone()));
    let mut shrinks = value.shrink();
    for shrinks.advance |elt| {
        let elt_cpy = elt.clone();
        if !prop(elt) {
            if cfg.verbose { println(fmt!("Shrunk to: %?", &elt_cpy)); }
            return quick_shrink(cfg, elt_cpy, prop);
        }
    }
    if cfg.verbose {
        println(fmt!("Shrink finished: %?", &value));
    }
    value
}

pub fn quick_check_occurs<A: Arbitrary>(cfg: QConfig, name: &str, prop: &fn(A) -> bool) {
    let mut n = 0u;
    for std::uint::range(0, cfg.trials) |i| {
        n += 1;
        let value = arbitrary(cfg.size + if cfg.grow { i / 8 } else { 0 });
        if prop(value) {
            if cfg.verbose {
                println(fmt!("qc %s: occured (%u trials)", name, n));
            }
            break;
        }
    }
    if n >= cfg.trials {
        fail!(fmt!("qc %s: could not to reproduce", name));
    }
}

pub macro_rules! quick_check(
    ($qc_property:expr) => (
        quick_check!(config, $qc_property)
    );
    ($qc_config:expr, $qc_property:expr) => ({
        quick_check(
            fmt!("%s\n%s:%u", stringify!($qc_property), file!(), line!()),
            $qc_config,
            $qc_property);
    })
)

pub macro_rules! quick_check_occurs(
    ($qc_property:expr) => (
        quick_check_occurs!(config.trials(config.trials * 4), $qc_property)
    );
    ($qc_config:expr, $qc_property:expr) => ({
        quick_check_occurs($qc_config,
            fmt!("%s:%u", file!(), line!()), $qc_property);
    })
)

/// Example of how to implement Arbitrary
#[deriving(Clone)]
enum UserType<T> {
    Nothing,
    Blob(int, ~str),
    Blub(~[T]),
}

impl<T: Clone + Arbitrary> Arbitrary for UserType<T> {
    fn arbitrary(sz: uint) -> UserType<T> {
        let x: u8 = std::rand::random();
        match x % 3 {
            0 => Nothing,
            1 => Blob(arbitrary(sz), arbitrary(sz)),
            _ => Blub(arbitrary(sz)),
        }
    }

}

impl Shrink for SmallN {
    fn shrink(&self) -> Lazy<SmallN> {
        do Lazy::create |L| {
            L.push_map((**self).shrink(), |x| SmallN(x));
        }
    }
}

/// Example of how to implement Arbitrary and Shrink
#[deriving(Clone)]
enum UserTree<T> {
    Nil,
    Node(T, ~UserTree<T>, ~UserTree<T>)
}

impl<T: Clone + Arbitrary> Arbitrary for UserTree<T> {
    fn arbitrary(sz: uint) -> UserTree<T> {
        let rint: u8 = std::rand::random();
        if sz == 0 || rint % 4 == 0 {
            Nil
        } else {
            Node(arbitrary(sz), ~arbitrary(sz/2), ~arbitrary(sz/2))
        }
    }
}

impl<T: Owned + Clone + Shrink> Shrink for UserTree<T> {
    fn shrink(&self) -> Lazy<UserTree<T>> {
        do Lazy::create |L| {
            match self.clone() {
                Nil => {}
                Node(x, l, r) => {
                    L.push(Nil);
                    L.push_map((x, l, r).shrink(), |(a, b, c)| Node(a, b, c));
                }
            }
        }
    }
}


#[test]
fn test_qc_basic() {
    let mut n = 0;
    quick_check!(|_: int| { n += 1; true} );
    assert_eq!(n, config.trials);

    let mut m = 0;
    quick_check_occurs!(|_: int| { m += 1; m == 20 });
    assert_eq!(m, 20);
}

#[test]
#[should_fail]
fn test_qc_fail() {
    quick_check!(|_: ()| false);
}

#[test]
#[should_fail]
fn test_qc_occurs_fail() {
    quick_check_occurs!(|s: ~str| s.len() == -1);
}

#[test]
fn test_qc_func() {
    let mut n = 0;
    quick_check("7 trials", config.trials(7), |_: int| { n += 1; true} );
    assert_eq!(n, 7);
}

#[test]
fn test_qc_config() {
    quick_check!(config.trials(0), |_: ()| false );
    quick_check!(config.trials(1), |_: ()| true );

    let mut n = 0;
    quick_check!(config.trials(7), |_: ()| { n += 1; true} );
    assert_eq!(n, 7);

    quick_check_occurs!(config.size(1000), |n: SmallN| *n > 1000);
}


#[test]
fn test_qc_smalln() {
    quick_check_occurs!(|n: SmallN| *n == 0);
    quick_check_occurs!(|n: SmallN| *n == 1);
    quick_check_occurs!(|n: SmallN| *n > 10);
}

#[test]
fn test_qc_shrink() {
    /* Test minimal shrinks with false props */
    let v = SmallN(100);
    let shrink = quick_shrink(config, v, |_| false);
    assert_eq!(*shrink, 0);

    let v = 20000000u;
    let shrink = quick_shrink(config, v, |x| x < 1200301);
    assert_eq!(shrink, 1200301);

    let s = ~[0, 1, 1, 2, 1, 0, 1, 0, 1];
    let shrink = quick_shrink(config, s, |_| false);
    assert_eq!(shrink, ~[]);

    /* Make sure we can shrink nested containers */
    let v = Some(~[Some(~"hi"), None, Some(~""), Some(~"long text from me")]);
    let shrink = quick_shrink(config, v, |_| false);
    assert_eq!(shrink, None);

    let s = ~[Some(~"hi"), None, Some(~"more"), None];
    assert_eq!(quick_shrink(config, s, |v| !v.iter().filter_map(|&x| x).any_(|s| s.contains_char('e'))),
        ~[Some(~"e")]);

    let s = ~"boots are made for walking";
    assert_eq!(quick_shrink(config, s, |v| v.iter().count(|x| x == 'a') <= 1),
        ~"aa");

    let s = ~[0, 1, 1, 2, 1, 0, 1, 0, 1];
    let sum = |v: ~[int]| v.iter().fold(0, |a, &b| a + b);
    let shrink = quick_shrink(config, s, |v| sum(v) < 3);
    assert_eq!(sum(shrink), 3);

    let s = (~"more meat", ~"beef");
    let shrink = quick_shrink(config, s, |(a, b)| !(a.contains_char('e') && b.contains_char('e')));
    assert_eq!(shrink, (~"e", ~"e"));

    let s = (SmallN(1), SmallN(10), SmallN(3));
    let shrink = quick_shrink(config, s, |(a, b, c)| *a + *b + *c == 0);
    assert_eq!(shrink, (SmallN(0), SmallN(0), SmallN(1)));

    /* test the biggest supported tuple */
    let t: (uint, (), ~[u8], Option<bool>, u8, ~str) = arbitrary(config.size);
    let shrink = quick_shrink(config, t, |_| false);
    assert_eq!(shrink, (0, (), ~[], None, 0, ~""));
}

#[test]
#[should_fail]
fn test_qc_tree() {
    quick_check!(config.size(7),
        |u: UserTree<u8>| match u {
            Node(x, ~Node(y, _, _), ~Nil) => (x ^ y) & 0x13 == 0,
            _ => true,
        });
    /* crashing..
    fail!("missing test");
    */
}

#[test]
#[should_fail]
fn test_qc_shrink_fail() {
    quick_check!(config.verbose(false).trials(100),
        |(a, b): (~str, ~str)| !(a.contains_char('e') || b.contains_char('e')));
}


#[deriving(Rand, Clone)]
struct Test_Foo { x: float, u: int }

#[test]
fn test_qc_random() {
    /*
    quick_check!(|_: Random<Test_Foo>| true);
    */
}

#[test]
fn test_qc_containers() {
    quick_check_occurs!(|s: Option<int>| s.is_none());
    quick_check_occurs!(|s: Option<int>| s.is_some());

    quick_check_occurs!(|v: ~[u8]| v.len() == 0);
    quick_check_occurs!(|v: ~[u8]| v.len() == 1);
    quick_check_occurs!(|v: ~[u8]| v.len() > 10);
    quick_check_occurs!(config.size(100), |v: ~[u8]| v.len() > 100);

    quick_check!(|s: ~str| s.is_ascii());

    quick_check_occurs!(|s: Unicode| s.len() > 0 && s.is_ascii());
    quick_check_occurs!(|s: Unicode| !s.is_ascii());
}

#[test]
#[should_fail]
fn test_invalid_utf8() {
    /* Demonstrate is_utf8 accepts some invalid utf-8 */
    quick_check!(config.verbose(true).grow(false).trials(5000), |v: ~[u8]| {
        if std::str::is_utf8(v) {
            v.iter().all(|&c| c != 192 && c != 193 && (c < 245))
        } else { true }
    });
}

#[test]
fn test_str() {
    quick_check!(|s: ~[char]| {
        let ss = std::str::from_chars(s);
        std::str::is_utf8(ss.as_bytes())
    });

    //assert!(!std::str::is_utf8(&[69, 70, 119, 213, 182, 73, 244, 145, 164, 184]));

    quick_check!(|s: ~str| {
        let bs = s.as_bytes_with_null();
        bs.len() > 0 && bs[bs.len()-1] == 0
    });
}

#[test]
fn test_random_stuff() {
    quick_check!(|v: ~[int]| { (v.head_opt().is_some()) == (v.len() > 0) });
    quick_check!(|v: ~[~str]| v.head_opt() == v.iter().next());

    /*
    quick_check!(|(v, n): (~[i8], SmallN)| {
        v.iter().take_(*n).len_() == v.len().min(&*n)
    });
    */

    quick_check!(|v: ~[Option<i8>]| { v == v.iter().transform(|&elt| elt).collect() });

    quick_check!(|v: ~[~str]| { v == v.clone() });

    /* Check that chain is correct length */
    quick_check!(|(x,y): (~[u8], ~[u8])| {
        x.len() + y.len() == x.iter().chain_(y.iter()).len_()
    });
    /* Check that chain has the right elements */
    quick_check!(|(x,y): (~[u8], ~[u8])| {
        x.iter().chain_(y.iter()).skip(x.len()).zip(y.iter()).all(|(a, b)| a == b)
    });

    /* Check that enumerate is indexing correctly */
    quick_check!(|x: ~[int]| {
        x.iter().enumerate().all(|(i, &elt)| x[i] == elt)
    });

    quick_check!(|(x,y): (~[u8], ~[u8])| {
        x.iter().zip(y.iter()).len_() == x.len().min(&y.len())
    });

    quick_check!(|(x,y): (~[u8], ~[u8])| {
        let v = [&x, &y];
        let xs = v.iter().flat_map_(|a| a.iter());
        let ys: ~[u8] = xs.transform(|&x: &u8| x).collect();
        ys.iter().zip(x.iter().chain_(y.iter())).all(|(a, b)| *a == *b) &&
            ys.len() == x.len() + y.len()
    });
}
