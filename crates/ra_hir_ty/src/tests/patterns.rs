use super::{infer, infer_with_mismatches};
use insta::assert_snapshot;
use test_utils::covers;

#[test]
fn infer_pattern() {
    assert_snapshot!(
        infer(r#"
fn test(x: &i32) {
    let y = x;
    let &z = x;
    let a = z;
    let (c, d) = (1, "hello");

    for (e, f) in some_iter {
        let g = e;
    }

    if let [val] = opt {
        let h = val;
    }

    let lambda = |a: u64, b, c: i32| { a + b; c };

    let ref ref_to_x = x;
    let mut mut_x = x;
    let ref mut mut_ref_to_x = x;
    let k = mut_ref_to_x;
}
"#),
        @r###"
    [9; 10) 'x': &i32
    [18; 369) '{     ...o_x; }': ()
    [28; 29) 'y': &i32
    [32; 33) 'x': &i32
    [43; 45) '&z': &i32
    [44; 45) 'z': i32
    [48; 49) 'x': &i32
    [59; 60) 'a': i32
    [63; 64) 'z': i32
    [74; 80) '(c, d)': (i32, &str)
    [75; 76) 'c': i32
    [78; 79) 'd': &str
    [83; 95) '(1, "hello")': (i32, &str)
    [84; 85) '1': i32
    [87; 94) '"hello"': &str
    [102; 152) 'for (e...     }': ()
    [106; 112) '(e, f)': (?, ?)
    [107; 108) 'e': ?
    [110; 111) 'f': ?
    [116; 125) 'some_iter': ?
    [126; 152) '{     ...     }': ()
    [140; 141) 'g': ?
    [144; 145) 'e': ?
    [158; 205) 'if let...     }': ()
    [165; 170) '[val]': [?]
    [166; 169) 'val': ?
    [173; 176) 'opt': [?]
    [177; 205) '{     ...     }': ()
    [191; 192) 'h': ?
    [195; 198) 'val': ?
    [215; 221) 'lambda': |u64, u64, i32| -> i32
    [224; 256) '|a: u6...b; c }': |u64, u64, i32| -> i32
    [225; 226) 'a': u64
    [233; 234) 'b': u64
    [236; 237) 'c': i32
    [244; 256) '{ a + b; c }': i32
    [246; 247) 'a': u64
    [246; 251) 'a + b': u64
    [250; 251) 'b': u64
    [253; 254) 'c': i32
    [267; 279) 'ref ref_to_x': &&i32
    [282; 283) 'x': &i32
    [293; 302) 'mut mut_x': &i32
    [305; 306) 'x': &i32
    [316; 336) 'ref mu...f_to_x': &mut &i32
    [339; 340) 'x': &i32
    [350; 351) 'k': &mut &i32
    [354; 366) 'mut_ref_to_x': &mut &i32
    "###
    );
}

#[test]
fn infer_pattern_match_ergonomics() {
    assert_snapshot!(
        infer(r#"
struct A<T>(T);

fn test() {
    let A(n) = &A(1);
    let A(n) = &mut A(1);
}
"#),
    @r###"
    [28; 79) '{     ...(1); }': ()
    [38; 42) 'A(n)': A<i32>
    [40; 41) 'n': &i32
    [45; 50) '&A(1)': &A<i32>
    [46; 47) 'A': A<i32>(i32) -> A<i32>
    [46; 50) 'A(1)': A<i32>
    [48; 49) '1': i32
    [60; 64) 'A(n)': A<i32>
    [62; 63) 'n': &mut i32
    [67; 76) '&mut A(1)': &mut A<i32>
    [72; 73) 'A': A<i32>(i32) -> A<i32>
    [72; 76) 'A(1)': A<i32>
    [74; 75) '1': i32
    "###
    );
}

#[test]
fn infer_pattern_match_ergonomics_ref() {
    covers!(match_ergonomics_ref);
    assert_snapshot!(
        infer(r#"
fn test() {
    let v = &(1, &2);
    let (_, &w) = v;
}
"#),
    @r###"
    [11; 57) '{     ...= v; }': ()
    [21; 22) 'v': &(i32, &i32)
    [25; 33) '&(1, &2)': &(i32, &i32)
    [26; 33) '(1, &2)': (i32, &i32)
    [27; 28) '1': i32
    [30; 32) '&2': &i32
    [31; 32) '2': i32
    [43; 50) '(_, &w)': (i32, &i32)
    [44; 45) '_': i32
    [47; 49) '&w': &i32
    [48; 49) 'w': i32
    [53; 54) 'v': &(i32, &i32)
    "###
    );
}

#[test]
fn infer_pattern_match_slice() {
    assert_snapshot!(
        infer(r#"
fn test() {
    let slice: &[f64] = &[0.0];
    match slice {
        &[] => {},
        &[a] => {
            a;
        },
        &[b, c] => {
            b;
            c;
        }
        _ => {}
    }
}
"#),
    @r###"
    [11; 210) '{     ...   } }': ()
    [21; 26) 'slice': &[f64]
    [37; 43) '&[0.0]': &[f64; _]
    [38; 43) '[0.0]': [f64; _]
    [39; 42) '0.0': f64
    [49; 208) 'match ...     }': ()
    [55; 60) 'slice': &[f64]
    [71; 74) '&[]': &[f64]
    [72; 74) '[]': [f64]
    [78; 80) '{}': ()
    [90; 94) '&[a]': &[f64]
    [91; 94) '[a]': [f64]
    [92; 93) 'a': f64
    [98; 124) '{     ...     }': ()
    [112; 113) 'a': f64
    [134; 141) '&[b, c]': &[f64]
    [135; 141) '[b, c]': [f64]
    [136; 137) 'b': f64
    [139; 140) 'c': f64
    [145; 186) '{     ...     }': ()
    [159; 160) 'b': f64
    [174; 175) 'c': f64
    [195; 196) '_': &[f64]
    [200; 202) '{}': ()
    "###
    );
}

#[test]
fn infer_pattern_match_arr() {
    assert_snapshot!(
        infer(r#"
fn test() {
    let arr: [f64; 2] = [0.0, 1.0];
    match arr {
        [1.0, a] => {
            a;
        },
        [b, c] => {
            b;
            c;
        }
    }
}
"#),
    @r###"
    [11; 180) '{     ...   } }': ()
    [21; 24) 'arr': [f64; _]
    [37; 47) '[0.0, 1.0]': [f64; _]
    [38; 41) '0.0': f64
    [43; 46) '1.0': f64
    [53; 178) 'match ...     }': ()
    [59; 62) 'arr': [f64; _]
    [73; 81) '[1.0, a]': [f64; _]
    [74; 77) '1.0': f64
    [79; 80) 'a': f64
    [85; 111) '{     ...     }': ()
    [99; 100) 'a': f64
    [121; 127) '[b, c]': [f64; _]
    [122; 123) 'b': f64
    [125; 126) 'c': f64
    [131; 172) '{     ...     }': ()
    [145; 146) 'b': f64
    [160; 161) 'c': f64
    "###
    );
}

#[test]
fn infer_adt_pattern() {
    assert_snapshot!(
        infer(r#"
enum E {
    A { x: usize },
    B
}

struct S(u32, E);

fn test() {
    let e = E::A { x: 3 };

    let S(y, z) = foo;
    let E::A { x: new_var } = e;

    match e {
        E::A { x } => x,
        E::B if foo => 1,
        E::B => 10,
    };

    let ref d @ E::A { .. } = e;
    d;
}
"#),
        @r###"
    [68; 289) '{     ...  d; }': ()
    [78; 79) 'e': E
    [82; 95) 'E::A { x: 3 }': E
    [92; 93) '3': usize
    [106; 113) 'S(y, z)': S
    [108; 109) 'y': u32
    [111; 112) 'z': E
    [116; 119) 'foo': S
    [129; 148) 'E::A {..._var }': E
    [139; 146) 'new_var': usize
    [151; 152) 'e': E
    [159; 245) 'match ...     }': usize
    [165; 166) 'e': E
    [177; 187) 'E::A { x }': E
    [184; 185) 'x': usize
    [191; 192) 'x': usize
    [202; 206) 'E::B': E
    [210; 213) 'foo': bool
    [217; 218) '1': usize
    [228; 232) 'E::B': E
    [236; 238) '10': usize
    [256; 275) 'ref d ...{ .. }': &E
    [264; 275) 'E::A { .. }': E
    [278; 279) 'e': E
    [285; 286) 'd': &E
    "###
    );
}

#[test]
fn infer_generics_in_patterns() {
    assert_snapshot!(
        infer(r#"
struct A<T> {
    x: T,
}

enum Option<T> {
    Some(T),
    None,
}

fn test(a1: A<u32>, o: Option<u64>) {
    let A { x: x2 } = a1;
    let A::<i64> { x: x3 } = A { x: 1 };
    match o {
        Option::Some(t) => t,
        _ => 1,
    };
}
"#),
        @r###"
    [79; 81) 'a1': A<u32>
    [91; 92) 'o': Option<u64>
    [107; 244) '{     ...  }; }': ()
    [117; 128) 'A { x: x2 }': A<u32>
    [124; 126) 'x2': u32
    [131; 133) 'a1': A<u32>
    [143; 161) 'A::<i6...: x3 }': A<i64>
    [157; 159) 'x3': i64
    [164; 174) 'A { x: 1 }': A<i64>
    [171; 172) '1': i64
    [180; 241) 'match ...     }': u64
    [186; 187) 'o': Option<u64>
    [198; 213) 'Option::Some(t)': Option<u64>
    [211; 212) 't': u64
    [217; 218) 't': u64
    [228; 229) '_': Option<u64>
    [233; 234) '1': u64
    "###
    );
}

#[test]
fn infer_const_pattern() {
    assert_snapshot!(
        infer_with_mismatches(r#"
enum Option<T> { None }
use Option::None;
struct Foo;
const Bar: usize = 1;

fn test() {
    let a: Option<u32> = None;
    let b: Option<i64> = match a {
        None => None,
    };
    let _: () = match () { Foo => Foo }; // Expected mismatch
    let _: () = match () { Bar => Bar }; // Expected mismatch
}
"#, true),
        @r###"
    [74; 75) '1': usize
    [88; 310) '{     ...atch }': ()
    [98; 99) 'a': Option<u32>
    [115; 119) 'None': Option<u32>
    [129; 130) 'b': Option<i64>
    [146; 183) 'match ...     }': Option<i64>
    [152; 153) 'a': Option<u32>
    [164; 168) 'None': Option<u32>
    [172; 176) 'None': Option<i64>
    [193; 194) '_': ()
    [201; 224) 'match ... Foo }': Foo
    [207; 209) '()': ()
    [212; 215) 'Foo': Foo
    [219; 222) 'Foo': Foo
    [255; 256) '_': ()
    [263; 286) 'match ... Bar }': usize
    [269; 271) '()': ()
    [274; 277) 'Bar': usize
    [281; 284) 'Bar': usize
    [201; 224): expected (), got Foo
    [263; 286): expected (), got usize
    "###
    );
}
