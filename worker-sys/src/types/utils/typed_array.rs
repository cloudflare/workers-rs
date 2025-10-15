#[macro_export]
macro_rules! typed_array {
    ($name:ident, $type:ty) => {
        #[wasm_bindgen]
        extern "C" {
            #[wasm_bindgen (extends = :: js_sys :: Array)]
            #[derive(Debug, Clone, PartialEq, Eq)]
            pub type $name;

            #[wasm_bindgen(constructor, js_class = Array)]
            fn new() -> $name;

            #[wasm_bindgen(constructor)]
            pub fn new_with_length(len: u32) -> $name;

            #[wasm_bindgen(method)]
            pub fn at(this: &$name, index: i32) -> Option<$type>;

            #[wasm_bindgen(method, structural, indexing_getter)]
            pub fn get(this: &$name, index: u32) -> Option<$type>;

            #[wasm_bindgen(method, structural, indexing_setter)]
            pub fn set(this: &$name, index: u32, value: $type);

            #[wasm_bindgen(method, structural, indexing_deleter)]
            pub fn delete(this: &$name, index: u32);

            #[wasm_bindgen(static_method_of = $name)]
            pub fn from(val: &$name) -> $name;

            #[wasm_bindgen(method, js_name = copyWithin)]
            pub fn copy_within(this: &$name, target: i32, start: i32, end: i32) -> $name;

            #[wasm_bindgen(method)]
            pub fn concat(this: &$name, array: &$name) -> $name;

            #[wasm_bindgen(method)]
            pub fn every(
                this: &$name,
                predicate: &mut dyn FnMut($type, u32, $name) -> bool,
            ) -> bool;

            #[wasm_bindgen(method)]
            pub fn fill(this: &$name, value: &$type, start: u32, end: u32) -> $name;

            #[wasm_bindgen(method)]
            pub fn filter(
                this: &$name,
                predicate: &mut dyn FnMut($type, u32, $name) -> bool,
            ) -> $name;

            #[wasm_bindgen(method)]
            pub fn find(
                this: &$name,
                predicate: &mut dyn FnMut($type, u32, $name) -> bool,
            ) -> $name;

            #[wasm_bindgen(method, js_name = findIndex)]
            pub fn find_index(
                this: &$name,
                predicate: &mut dyn FnMut($type, u32, $name) -> bool,
            ) -> i32;

            #[wasm_bindgen(method, js_name = findLast)]
            pub fn find_last(
                this: &$name,
                predicate: &mut dyn FnMut($type, u32, $name) -> bool,
            ) -> $type;

            #[wasm_bindgen(method, js_name = findLastIndex)]
            pub fn find_last_index(
                this: &$name,
                predicate: &mut dyn FnMut($type, u32, $name) -> bool,
            ) -> i32;

            #[wasm_bindgen(method)]
            pub fn flat(this: &$name, depth: i32) -> $name;

            #[wasm_bindgen(method, js_name = flatMap)]
            pub fn flat_map(
                this: &$name,
                callback: &mut dyn FnMut($type, u32, $name) -> Vec<$type>,
            ) -> $name;

            #[wasm_bindgen(method, js_name = forEach)]
            pub fn for_each(this: &$name, callback: &mut dyn FnMut($type, u32, $name));

            #[wasm_bindgen(method)]
            pub fn includes(this: &$name, value: &$type, from_index: i32) -> bool;

            #[wasm_bindgen(method, js_name = indexOf)]
            pub fn index_of(this: &$name, value: &$type, from_index: i32) -> i32;

            #[wasm_bindgen(static_method_of = $name, js_name = isArray)]
            pub fn is_array(value: &$type) -> bool;

            #[wasm_bindgen(method)]
            pub fn join(this: &$name, delimiter: &str) -> ::js_sys::JsString;

            #[wasm_bindgen(method, js_name = lastIndexOf)]
            pub fn last_index_of(this: &$name, value: &$type, from_index: i32) -> i32;

            #[wasm_bindgen(method, getter, structural)]
            pub fn length(this: &$name) -> u32;

            #[wasm_bindgen(method, setter)]
            pub fn set_length(this: &$name, value: u32);

            #[wasm_bindgen(method)]
            pub fn map(
                this: &$name,
                predicate: &mut dyn FnMut($type, u32, $name) -> $type,
            ) -> $name;

            #[wasm_bindgen(static_method_of = $name, js_name = of)]
            pub fn of1(a: &$type) -> $name;

            #[wasm_bindgen(static_method_of = $name, js_name = of)]
            pub fn of2(a: &$type, b: &$type) -> $name;

            #[wasm_bindgen(static_method_of = $name, js_name = of)]
            pub fn of3(a: &$type, b: &$type, c: &$type) -> $name;

            #[wasm_bindgen(static_method_of = $name, js_name = of)]
            pub fn of4(a: &$type, b: &$type, c: &$type, d: &$type) -> $name;

            #[wasm_bindgen(static_method_of = $name, js_name = of)]
            pub fn of5(a: &$type, b: &$type, c: &$type, d: &$type, e: &$type) -> $name;

            #[wasm_bindgen(method)]
            pub fn pop(this: &$name) -> $type;

            #[wasm_bindgen(method)]
            pub fn push(this: &$name, value: &$type) -> u32;

            #[wasm_bindgen(method)]
            pub fn reduce(
                this: &$name,
                predicate: &mut dyn FnMut($type, $type, u32, $name) -> $type,
                initial_value: &$type,
            ) -> $type;

            #[wasm_bindgen(method, js_name = reduceRight)]
            pub fn reduce_right(
                this: &$name,
                predicate: &mut dyn FnMut($type, $type, u32, $name) -> $type,
                initial_value: &$type,
            ) -> $type;

            #[wasm_bindgen(method)]
            pub fn reverse(this: &$name) -> $name;

            #[wasm_bindgen(method)]
            pub fn shift(this: &$name) -> $type;

            #[wasm_bindgen(method)]
            pub fn slice(this: &$name, start: u32, end: u32) -> $name;

            #[wasm_bindgen(method)]
            pub fn some(this: &$name, predicate: &mut dyn FnMut($type) -> bool) -> bool;

            #[wasm_bindgen(method)]
            pub fn sort(this: &$name) -> $name;

            #[wasm_bindgen(method)]
            pub fn splice(this: &$name, start: u32, delete_count: u32, item: &$type) -> $name;

            #[wasm_bindgen(method, js_name = toLocaleString)]
            pub fn to_locale_string(
                this: &$name,
                locales: &$type,
                options: &$type,
            ) -> ::js_sys::JsString;

            #[wasm_bindgen(method, js_name = toString)]
            pub fn to_string(this: &$name) -> ::js_sys::JsString;

            #[wasm_bindgen(method)]
            pub fn unshift(this: &$name, value: &$type) -> u32;
        }
    };
}
