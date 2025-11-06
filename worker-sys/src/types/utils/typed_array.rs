pub(crate) trait TypedArray {
    type Item;

    fn new_with_length(len: u32) -> Self;

    fn push(&self, item: &Self::Item) -> u32;
}

#[allow(private_bounds)]
pub struct TypedArrayBuilder<'a, T: TypedArray> {
    item: Option<&'a T::Item>,
    builder: Option<&'a TypedArrayBuilder<'a, T>>,
    index: Option<u32>,
}

impl<'a, T: TypedArray> Default for TypedArrayBuilder<'a, T> {
    fn default() -> Self {
        Self {
            item: None,
            builder: None,
            index: Some(0),
        }
    }
}

#[allow(private_bounds)]
impl<'a, T: TypedArray> TypedArrayBuilder<'a, T> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn push(mut self, item: impl Into<&'a T::Item>) -> TypedArrayBuilder<'a, T> {
        TypedArrayBuilder {
            item: Some(item.into()),
            index: self.index.take().map(|x| x + 1),
            builder: self.builder.take(),
        }
    }

    pub fn build(self) -> T {
        let vec = T::new_with_length(self.index.unwrap());
        let mut builder_option = self.builder;
        let mut item_option = self.item;
        while let Some((item, builder)) = item_option.take().zip(builder_option.take()) {
            vec.push(item);
            builder_option = builder.builder
        }
        vec
    }
}

#[macro_export]
macro_rules! typed_array {
    ($name:ident, $type:ident) => {
        #[allow(non_snake_case)]
        mod $name {
            use super::$type;
            use ::wasm_bindgen::prelude::*;
            use $crate::utils::typed_array::TypedArray;

            #[wasm_bindgen]
            extern "C" {
                #[wasm_bindgen (extends = :: js_sys :: Array)]
                #[derive(Debug, Clone, PartialEq, Eq)]
                pub type $name;

                #[wasm_bindgen(constructor, js_class = Array)]
                fn new() -> $name;

                #[wasm_bindgen(constructor)]
                fn new_with_length(len: u32) -> $name;

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

            #[derive(Debug, Clone)]
            pub struct ArrayIntoIter {
                range: core::ops::Range<u32>,
                array: $name,
            }

            impl core::iter::Iterator for ArrayIntoIter {
                type Item = $type;

                fn next(&mut self) -> Option<Self::Item> {
                    let index = self.range.next()?;
                    self.array.get(index)
                }

                #[inline]
                fn size_hint(&self) -> (usize, Option<usize>) {
                    self.range.size_hint()
                }

                #[inline]
                fn count(self) -> usize
                where
                    Self: Sized,
                {
                    self.range.count()
                }

                #[inline]
                fn last(self) -> Option<Self::Item>
                where
                    Self: Sized,
                {
                    let Self { range, array } = self;
                    range.last().map(|index| array.get(index)).flatten()
                }

                #[inline]
                fn nth(&mut self, n: usize) -> Option<Self::Item> {
                    self.range
                        .nth(n)
                        .map(|index| self.array.get(index))
                        .flatten()
                }
            }

            impl core::iter::DoubleEndedIterator for ArrayIntoIter {
                fn next_back(&mut self) -> Option<Self::Item> {
                    let index = self.range.next_back()?;
                    self.array.get(index)
                }

                fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
                    self.range
                        .nth_back(n)
                        .map(|index| self.array.get(index))
                        .flatten()
                }
            }

            impl core::iter::FusedIterator for ArrayIntoIter {}

            impl core::iter::ExactSizeIterator for ArrayIntoIter {}

            #[derive(Debug, Clone)]
            pub struct ArrayIter<'a> {
                range: core::ops::Range<u32>,
                array: &'a $name,
            }

            impl core::iter::Iterator for ArrayIter<'_> {
                type Item = $type;

                fn next(&mut self) -> Option<Self::Item> {
                    let index = self.range.next()?;
                    self.array.get(index)
                }

                #[inline]
                fn size_hint(&self) -> (usize, Option<usize>) {
                    self.range.size_hint()
                }

                #[inline]
                fn count(self) -> usize
                where
                    Self: Sized,
                {
                    self.range.count()
                }

                #[inline]
                fn last(self) -> Option<Self::Item>
                where
                    Self: Sized,
                {
                    let Self { range, array } = self;
                    range.last().map(|index| array.get(index)).flatten()
                }

                #[inline]
                fn nth(&mut self, n: usize) -> Option<Self::Item> {
                    self.range
                        .nth(n)
                        .map(|index| self.array.get(index))
                        .flatten()
                }
            }

            impl core::iter::DoubleEndedIterator for ArrayIter<'_> {
                fn next_back(&mut self) -> Option<Self::Item> {
                    let index = self.range.next_back()?;
                    self.array.get(index)
                }

                fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
                    self.range
                        .nth_back(n)
                        .map(|index| self.array.get(index))
                        .flatten()
                }
            }

            impl core::iter::FusedIterator for ArrayIter<'_> {}

            impl core::iter::ExactSizeIterator for ArrayIter<'_> {}

            impl $name {
                /// Returns an iterator over the values of the JS array.
                pub fn iter(&self) -> ArrayIter<'_> {
                    ArrayIter {
                        range: 0..self.length(),
                        array: self,
                    }
                }

                /// Converts the JS array into a new Vec.
                pub fn to_vec(&self) -> Vec<Option<$type>> {
                    let len = self.length();

                    let mut output = Vec::with_capacity(len as usize);
                    for i in 0..len {
                        output.push(self.get(i));
                    }

                    output
                }
            }

            impl core::iter::IntoIterator for $name {
                type Item = $type;
                type IntoIter = ArrayIntoIter;

                fn into_iter(self) -> Self::IntoIter {
                    ArrayIntoIter {
                        range: 0..self.length(),
                        array: self,
                    }
                }
            }

            // TODO pre-initialize the Array with the correct length using TrustedLen
            impl core::iter::FromIterator<$type> for $name {
                fn from_iter<T>(iter: T) -> $name
                where
                    T: IntoIterator<Item = $type>,
                {
                    let mut out = $name::new();
                    out.extend(iter);
                    out
                }
            }

            impl core::iter::Extend<$type> for $name {
                fn extend<T>(&mut self, iter: T)
                where
                    T: IntoIterator<Item = $type>,
                {
                    for value in iter {
                        self.push(value.as_ref());
                    }
                }
            }

            impl TypedArray for $name {
                type Item = $type;

                fn new_with_length(len: u32) -> $name {
                    Self::new_with_length(len)
                }

                fn push(&self, item: &Self::Item) -> u32 {
                    self.push(&item)
                }
            }
        }
    };
}
