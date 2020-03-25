use insta::assert_snapshot;
use test_utils::covers;

use super::infer;
use crate::test_db::TestDB;
use ra_db::fixture::WithFixture;

#[test]
fn bug_484() {
    assert_snapshot!(
        infer(r#"
fn test() {
   let x = if true {};
}
"#),
        @r###"
    [11; 37) '{    l... {}; }': ()
    [20; 21) 'x': ()
    [24; 34) 'if true {}': ()
    [27; 31) 'true': bool
    [32; 34) '{}': ()
    "###
    );
}

#[test]
fn no_panic_on_field_of_enum() {
    assert_snapshot!(
        infer(r#"
enum X {}

fn test(x: X) {
    x.some_field;
}
"#),
        @r###"
    [20; 21) 'x': X
    [26; 47) '{     ...eld; }': ()
    [32; 33) 'x': X
    [32; 44) 'x.some_field': ?
    "###
    );
}

#[test]
fn bug_585() {
    assert_snapshot!(
        infer(r#"
fn test() {
    X {};
    match x {
        A::B {} => (),
        A::Y() => (),
    }
}
"#),
        @r###"
    [11; 89) '{     ...   } }': ()
    [17; 21) 'X {}': ?
    [27; 87) 'match ...     }': ()
    [33; 34) 'x': ?
    [45; 52) 'A::B {}': ?
    [56; 58) '()': ()
    [68; 74) 'A::Y()': ?
    [78; 80) '()': ()
    "###
    );
}

#[test]
fn bug_651() {
    assert_snapshot!(
        infer(r#"
fn quux() {
    let y = 92;
    1 + y;
}
"#),
        @r###"
    [11; 41) '{     ...+ y; }': ()
    [21; 22) 'y': i32
    [25; 27) '92': i32
    [33; 34) '1': i32
    [33; 38) '1 + y': i32
    [37; 38) 'y': i32
    "###
    );
}

#[test]
fn recursive_vars() {
    covers!(type_var_cycles_resolve_completely);
    covers!(type_var_cycles_resolve_as_possible);
    assert_snapshot!(
        infer(r#"
fn test() {
    let y = unknown;
    [y, &y];
}
"#),
        @r###"
    [11; 48) '{     ...&y]; }': ()
    [21; 22) 'y': &?
    [25; 32) 'unknown': &?
    [38; 45) '[y, &y]': [&&?; _]
    [39; 40) 'y': &?
    [42; 44) '&y': &&?
    [43; 44) 'y': &?
    "###
    );
}

#[test]
fn recursive_vars_2() {
    covers!(type_var_cycles_resolve_completely);
    covers!(type_var_cycles_resolve_as_possible);
    assert_snapshot!(
        infer(r#"
fn test() {
    let x = unknown;
    let y = unknown;
    [(x, y), (&y, &x)];
}
"#),
        @r###"
    [11; 80) '{     ...x)]; }': ()
    [21; 22) 'x': &&?
    [25; 32) 'unknown': &&?
    [42; 43) 'y': &&?
    [46; 53) 'unknown': &&?
    [59; 77) '[(x, y..., &x)]': [(&&&?, &&&?); _]
    [60; 66) '(x, y)': (&&&?, &&&?)
    [61; 62) 'x': &&?
    [64; 65) 'y': &&?
    [68; 76) '(&y, &x)': (&&&?, &&&?)
    [69; 71) '&y': &&&?
    [70; 71) 'y': &&?
    [73; 75) '&x': &&&?
    [74; 75) 'x': &&?
    "###
    );
}

#[test]
fn infer_std_crash_1() {
    // caused stack overflow, taken from std
    assert_snapshot!(
        infer(r#"
enum Maybe<T> {
    Real(T),
    Fake,
}

fn write() {
    match something_unknown {
        Maybe::Real(ref mut something) => (),
    }
}
"#),
        @r###"
    [54; 139) '{     ...   } }': ()
    [60; 137) 'match ...     }': ()
    [66; 83) 'someth...nknown': Maybe<?>
    [94; 124) 'Maybe:...thing)': Maybe<?>
    [106; 123) 'ref mu...ething': &mut ?
    [128; 130) '()': ()
    "###
    );
}

#[test]
fn infer_std_crash_2() {
    covers!(type_var_resolves_to_int_var);
    // caused "equating two type variables, ...", taken from std
    assert_snapshot!(
        infer(r#"
fn test_line_buffer() {
    &[0, b'\n', 1, b'\n'];
}
"#),
        @r###"
    [23; 53) '{     ...n']; }': ()
    [29; 50) '&[0, b...b'\n']': &[u8; _]
    [30; 50) '[0, b'...b'\n']': [u8; _]
    [31; 32) '0': u8
    [34; 39) 'b'\n'': u8
    [41; 42) '1': u8
    [44; 49) 'b'\n'': u8
    "###
    );
}

#[test]
fn infer_std_crash_3() {
    // taken from rustc
    assert_snapshot!(
        infer(r#"
pub fn compute() {
    match nope!() {
        SizeSkeleton::Pointer { non_zero: true, tail } => {}
    }
}
"#),
        @r###"
    [18; 108) '{     ...   } }': ()
    [24; 106) 'match ...     }': ()
    [30; 37) 'nope!()': ?
    [48; 94) 'SizeSk...tail }': ?
    [82; 86) 'true': ?
    [88; 92) 'tail': ?
    [98; 100) '{}': ()
    "###
    );
}

#[test]
fn infer_std_crash_4() {
    // taken from rustc
    assert_snapshot!(
        infer(r#"
pub fn primitive_type() {
    match *self {
        BorrowedRef { type_: Primitive(p), ..} => {},
    }
}
"#),
        @r###"
    [25; 106) '{     ...   } }': ()
    [31; 104) 'match ...     }': ()
    [37; 42) '*self': ?
    [38; 42) 'self': ?
    [53; 91) 'Borrow...), ..}': ?
    [74; 86) 'Primitive(p)': ?
    [84; 85) 'p': ?
    [95; 97) '{}': ()
    "###
    );
}

#[test]
fn infer_std_crash_5() {
    // taken from rustc
    assert_snapshot!(
        infer(r#"
fn extra_compiler_flags() {
    for content in doesnt_matter {
        let name = if doesnt_matter {
            first
        } else {
            &content
        };

        let content = if ICE_REPORT_COMPILER_FLAGS_STRIP_VALUE.contains(&name) {
            name
        } else {
            content
        };
    }
}
"#),
        @r###"
    [27; 323) '{     ...   } }': ()
    [33; 321) 'for co...     }': ()
    [37; 44) 'content': &?
    [48; 61) 'doesnt_matter': ?
    [62; 321) '{     ...     }': ()
    [76; 80) 'name': &&?
    [83; 167) 'if doe...     }': &&?
    [86; 99) 'doesnt_matter': bool
    [100; 129) '{     ...     }': &&?
    [114; 119) 'first': &&?
    [135; 167) '{     ...     }': &&?
    [149; 157) '&content': &&?
    [150; 157) 'content': &?
    [182; 189) 'content': &?
    [192; 314) 'if ICE...     }': &?
    [195; 232) 'ICE_RE..._VALUE': ?
    [195; 248) 'ICE_RE...&name)': bool
    [242; 247) '&name': &&&?
    [243; 247) 'name': &&?
    [249; 277) '{     ...     }': &&?
    [263; 267) 'name': &&?
    [283; 314) '{     ...     }': &?
    [297; 304) 'content': &?
    "###
    );
}

#[test]
fn infer_nested_generics_crash() {
    // another crash found typechecking rustc
    assert_snapshot!(
        infer(r#"
struct Canonical<V> {
    value: V,
}
struct QueryResponse<V> {
    value: V,
}
fn test<R>(query_response: Canonical<QueryResponse<R>>) {
    &query_response.value;
}
"#),
        @r###"
    [92; 106) 'query_response': Canonical<QueryResponse<R>>
    [137; 167) '{     ...lue; }': ()
    [143; 164) '&query....value': &QueryResponse<R>
    [144; 158) 'query_response': Canonical<QueryResponse<R>>
    [144; 164) 'query_....value': QueryResponse<R>
    "###
    );
}

#[test]
fn infer_paren_macro_call() {
    assert_snapshot!(
        infer(r#"
macro_rules! bar { () => {0u32} }
fn test() {
    let a = (bar!());
}
"#),
        @r###"
    ![0; 4) '0u32': u32
    [45; 70) '{     ...()); }': ()
    [55; 56) 'a': u32
        "###
    );
}

#[test]
fn bug_1030() {
    assert_snapshot!(infer(r#"
struct HashSet<T, H>;
struct FxHasher;
type FxHashSet<T> = HashSet<T, FxHasher>;

impl<T, H> HashSet<T, H> {
    fn default() -> HashSet<T, H> {}
}

pub fn main_loop() {
    FxHashSet::default();
}
"#),
    @r###"
    [144; 146) '{}': ()
    [169; 198) '{     ...t(); }': ()
    [175; 193) 'FxHash...efault': fn default<?, FxHasher>() -> HashSet<?, FxHasher>
    [175; 195) 'FxHash...ault()': HashSet<?, FxHasher>
    "###
    );
}

#[test]
fn issue_2669() {
    assert_snapshot!(
        infer(
            r#"trait A {}
    trait Write {}
    struct Response<T> {}

    trait D {
        fn foo();
    }

    impl<T:A> D for Response<T> {
        fn foo() {
            end();
            fn end<W: Write>() {
                let _x: T =  loop {};
            }
        }
    }"#
        ),
        @r###"
    [147; 262) '{     ...     }': ()
    [161; 164) 'end': fn end<?>() -> ()
    [161; 166) 'end()': ()
    [199; 252) '{     ...     }': ()
    [221; 223) '_x': !
    [230; 237) 'loop {}': !
    [235; 237) '{}': ()
    "###
    )
}

#[test]
fn issue_2705() {
    assert_snapshot!(
        infer(r#"
trait Trait {}
fn test() {
    <Trait<u32>>::foo()
}
"#),
        @r###"
    [26; 53) '{     ...oo() }': ()
    [32; 49) '<Trait...>::foo': ?
    [32; 51) '<Trait...:foo()': ()
    "###
    );
}

#[test]
fn issue_2683_chars_impl() {
    let (db, pos) = TestDB::with_position(
        r#"
//- /main.rs crate:main deps:std
fn test() {
    let chars: std::str::Chars<'_>;
    (chars.next(), chars.nth(1))<|>;
}

//- /std.rs crate:std
#[prelude_import]
use prelude::*;

pub mod prelude {
    pub use crate::iter::Iterator;
    pub use crate::option::Option;
}

pub mod iter {
    pub use self::traits::Iterator;
    pub mod traits {
        pub use self::iterator::Iterator;

        pub mod iterator {
            pub trait Iterator {
                type Item;
                fn next(&mut self) -> Option<Self::Item>;
                fn nth(&mut self, n: usize) -> Option<Self::Item> {}
            }
        }
    }
}

pub mod option {
    pub enum Option<T> {}
}

pub mod str {
    pub struct Chars<'a> {}
    impl<'a> Iterator for Chars<'a> {
        type Item = char;
        fn next(&mut self) -> Option<char> {}
    }
}
"#,
    );

    // should be Option<char>, but currently not because of Chalk ambiguity problem
    assert_eq!("(Option<?>, Option<?>)", super::type_at_pos(&db, pos));
}

#[test]
fn issue_3642_bad_macro_stackover() {
    let (db, pos) = TestDB::with_position(
        r#"
//- /main.rs
#[macro_export]
macro_rules! match_ast {
    (match $node:ident { $($tt:tt)* }) => { match_ast!(match ($node) { $($tt)* }) };

    (match ($node:expr) {
        $( ast::$ast:ident($it:ident) => $res:expr, )*
        _ => $catch_all:expr $(,)?
    }) => {{
        $( if let Some($it) = ast::$ast::cast($node.clone()) { $res } else )*
        { $catch_all }
    }};
}

fn main() {
    let anchor<|> = match_ast! {
        match parent {
            as => {},
            _ => return None
        }
    };
}"#,
    );

    assert_eq!("()", super::type_at_pos(&db, pos));
}
