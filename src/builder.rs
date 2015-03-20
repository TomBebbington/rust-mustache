use std::cell::RefCell;
use std::collections::HashMap;
use serialize::Encodable;
use serialize::Encoder as Encoderer;

use encoder;
use encoder::Encoder;
use error::Error;
use data::Data;

/// `MapBuilder` is a helper type that construct `Data` types.
pub struct MapBuilder {
    data: HashMap<String, Data>,
}

impl MapBuilder {
    /// Create a `MapBuilder`
    #[inline]
    pub fn new() -> MapBuilder {
        MapBuilder {
            data: HashMap::new(),
        }
    }

    /// Add an `Encodable` to the `MapBuilder`.
    ///
    /// ```rust
    /// use mustache::MapBuilder;
    /// let data = MapBuilder::new()
    ///     .insert("name", &"Jane Austen").unwrap()
    ///     .insert("age", &41u).unwrap()
    ///     .build();
    /// ```
    #[inline]
    pub fn insert<
        T: Encodable,
        E: Encoderer<Error = Error>
    >(self, key: String, value: &T) -> Result<MapBuilder, Error> {
        let MapBuilder { mut data } = self;
        let value = try!(encoder::encode(value));
        data.insert(key, value);
        Ok(MapBuilder { data: data })
    }

    /// Add a `String` to the `MapBuilder`.
    ///
    /// ```rust
    /// use mustache::MapBuilder;
    /// let data = MapBuilder::new()
    ///     .insert_str("name".to_string(), "Jane Austen".to_string())
    ///     .build();
    /// ```
    #[inline]
    pub fn insert_str(self, key: String, value: String) -> MapBuilder {
        let MapBuilder { mut data } = self;
        data.insert(key, Data::Str(value.to_string()));
        MapBuilder { data: data }
    }

    /// Add a `bool` to the `MapBuilder`.
    ///
    /// ```rust
    /// use mustache::MapBuilder;
    /// let data = MapBuilder::new()
    ///     .insert_bool("show".to_string(), true)
    ///     .build();
    /// ```
    #[inline]
    pub fn insert_bool(self, key: String, value: bool) -> MapBuilder {
        let MapBuilder { mut data } = self;
        data.insert(key, Data::Bool(value));
        MapBuilder { data: data }
    }

    /// Add a `Vec` to the `MapBuilder`.
    ///
    /// ```rust
    /// use mustache::MapBuilder;
    /// let data = MapBuilder::new()
    ///     .insert_vec("authors", |builder| {
    ///         builder
    ///             .push_str("Jane Austen")
    ///             .push_str("Lewis Carroll")
    ///     })
    ///     .build();
    /// ```
    #[inline]
    pub fn insert_vec<F>(self, key: String, f: F) -> MapBuilder where F:FnOnce(VecBuilder) -> VecBuilder {
        let MapBuilder { mut data } = self;
        let builder = f(VecBuilder::new());
        data.insert(key, builder.build());
        MapBuilder { data: data }
    }

    /// Add a `Map` to the `MapBuilder`.
    ///
    /// ```rust
    /// use mustache::MapBuilder;
    /// let data = MapBuilder::new()
    ///     .insert_map("person1", |builder| {
    ///         builder
    ///             .insert_str("first_name", "Jane")
    ///             .insert_str("last_name", "Austen")
    ///     })
    ///     .insert_map("person2", |builder| {
    ///         builder
    ///             .insert_str("first_name", "Lewis")
    ///             .insert_str("last_name", "Carroll")
    ///     })
    ///     .build();
    /// ```
    #[inline]
    pub fn insert_map<F>(self, key: String, f: F) -> MapBuilder where F:FnOnce(MapBuilder) -> MapBuilder {
        let MapBuilder { mut data } = self;
        let builder = f(MapBuilder::new());
        data.insert(key, builder.build());
        MapBuilder { data: data }
    }

    /// Add a function to the `MapBuilder`.
    ///
    /// ```rust
    /// use mustache::MapBuilder;
    /// let mut count = 0;
    /// let data = MapBuilder::new()
    ///     .insert_fn("increment", |_| {
    ///         count += 1u;
    ///         count.to_string()
    ///     })
    ///     .build();
    /// ```
    #[inline]
    pub fn insert_fn<F>(self, key: String, f: F) -> MapBuilder where F:Fn(String) -> String + 'static {
        let MapBuilder { mut data } = self;
        data.insert(key, Data::Fun(RefCell::new(Box::new(f) as Box<Fn(String) -> String>)));
        MapBuilder { data: data }
    }

    /// Return the built `Data`.
    #[inline]
    pub fn build(self) -> Data {
        Data::Map(self.data)
    }
}

pub struct VecBuilder {
    data: Vec<Data>,
}

impl VecBuilder {
    /// Create a `VecBuilder`
    #[inline]
    pub fn new() -> VecBuilder {
        VecBuilder {
            data: Vec::new(),
        }
    }

    /// Add an `Encodable` to the `VecBuilder`.
    ///
    /// ```rust
    /// use mustache::{VecBuilder, Data};
    /// let data: Data = VecBuilder::new()
    ///     .push(& &"Jane Austen").unwrap()
    ///     .push(&41u).unwrap()
    ///     .build();
    /// ```
    #[inline]
    pub fn push<
        T: Encodable,
        E: Encoderer<Error = Error>
    >(self, value: &T) -> Result<VecBuilder, Error> {
        let VecBuilder { mut data } = self;
        let value = try!(encoder::encode(value));
        data.push(value);
        Ok(VecBuilder { data: data })
    }

    /// Add a `String` to the `VecBuilder`.
    ///
    /// ```rust
    /// use mustache::VecBuilder;
    /// let data = VecBuilder::new()
    ///     .push_str("Jane Austen".to_string())
    ///     .push_str("Lewis Carroll".to_string())
    ///     .build();
    /// ```
    #[inline]
    pub fn push_str(self, value: String) -> VecBuilder {
        let VecBuilder { mut data } = self;
        data.push(Data::Str(value));
        VecBuilder { data: data }
    }

    /// Add a `bool` to the `VecBuilder`.
    ///
    /// ```rust
    /// use mustache::VecBuilder;
    /// let data = VecBuilder::new()
    ///     .push_bool(false)
    ///     .push_bool(true)
    ///     .build();
    /// ```
    #[inline]
    pub fn push_bool(self, value: bool) -> VecBuilder {
        let VecBuilder { mut data } = self;
        data.push(Data::Bool(value));
        VecBuilder { data: data }
    }

    /// Add a `Vec` to the `MapBuilder`.
    ///
    /// ```rust
    /// use mustache::VecBuilder;
    /// let data = VecBuilder::new()
    ///     .push_vec(|builder| {
    ///         builder
    ///             .push_str("Jane Austen".to_string())
    ///             .push_str("Lewis Carroll".to_string())
    ///     })
    ///     .build();
    /// ```
    #[inline]
    pub fn push_vec<F>(self, f: F) -> VecBuilder where F: FnOnce(VecBuilder) -> VecBuilder {
        let VecBuilder { mut data } = self;
        let builder = f(VecBuilder::new());
        data.push(builder.build());
        VecBuilder { data: data }
    }

    /// Add a `Map` to the `VecBuilder`.
    ///
    /// ```rust
    /// use mustache::VecBuilder;
    /// let data = VecBuilder::new()
    ///     .push_map(|builder| {
    ///         builder
    ///             .insert_str("first_name".to_string(), "Jane".to_string())
    ///             .insert_str("last_name".to_string(), "Austen".to_string())
    ///     })
    ///     .push_map(|builder| {
    ///         builder
    ///             .insert_str("first_name".to_string(), "Lewis".to_string())
    ///             .insert_str("last_name".to_string(), "Carroll".to_string())
    ///     })
    ///     .build();
    /// ```
    #[inline]
    pub fn push_map<F>(self, f: F) -> VecBuilder where F:FnOnce(MapBuilder) -> MapBuilder {
        let VecBuilder { mut data } = self;
        let builder = f(MapBuilder::new());
        data.push(builder.build());
        VecBuilder { data: data }
    }

    /// Add a function to the `VecBuilder`.
    ///
    /// ```rust
    /// use mustache::VecBuilder;
    /// let mut count = 0;
    /// let data = VecBuilder::new()
    ///     .push_fn(|s| {
    ///         count += 1u;
    ///         s + count.to_string()
    ///     })
    ///     .build();
    /// ```
    #[inline]
    pub fn push_fn<F>(self, f: F) -> VecBuilder where F:Fn(String) -> String + 'static {
        let VecBuilder { mut data } = self;
        data.push(Data::Fun(RefCell::new(Box::new(f) as Box<Fn(String) -> String>)));
        VecBuilder { data: data }
    }

    #[inline]
    pub fn build(self) -> Data {
        Data::Vec(self.data)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use data::Data;
    use super::{MapBuilder, VecBuilder};

    #[test]
    fn test_empty_builders() {
        assert_eq!(
            MapBuilder::new().build(),
            Data::Map(HashMap::new()));

        assert_eq!(
            VecBuilder::new().build(),
            Data::Vec(Vec::new()));
    }

    #[test]
    fn test_builders() {
        let mut pride_and_prejudice = HashMap::new();
        pride_and_prejudice.insert("title".to_string(), Data::Str("Pride and Prejudice".to_string()));
        pride_and_prejudice.insert("publish_date".to_string(), Data::Str("1813".to_string()));

        let mut m = HashMap::new();
        m.insert("first_name".to_string(), Data::Str("Jane".to_string()));
        m.insert("last_name".to_string(), Data::Str("Austen".to_string()));
        m.insert("age".to_string(), Data::Str("41".to_string()));
        m.insert("died".to_string(), Data::Bool(true));
        m.insert("works".to_string(), Data::Vec(vec!(
            Data::Str("Sense and Sensibility".to_string()),
            Data::Map(pride_and_prejudice))));

        assert_eq!(
            MapBuilder::new()
                .insert_str("first_name", "Jane")
                .insert_str("last_name", "Austen")
                .insert("age", &41u).unwrap()
                .insert_bool("died", true)
                .insert_vec("works", |builder| {
                    builder
                        .push_str("Sense and Sensibility")
                        .push_map(|builder| {
                            builder
                                .insert_str("title", "Pride and Prejudice")
                                .insert("publish_date", &1813u).unwrap()
                        })
                })
                .build(),
            Data::Map(m));
    }

    #[test]
    fn test_map_fn_builder() {
        // We can't directly compare closures, so just make sure we thread
        // through the builder.

        let mut count = 0u;
        let data = MapBuilder::new()
            .insert_fn("count", |s| {
                count += 1u;
                s + count.to_string()
            })
            .build();

        match data {
            Data::Map(m) => {
                match *m.get("count").unwrap() {
                    Data::Fun(ref f) => {
                        let f = &mut *f.borrow_mut();
                        assert_eq!((*f)("count: ".to_string()), "count: 1".to_string());
                        assert_eq!((*f)("count: ".to_string()), "count: 2".to_string());
                        assert_eq!((*f)("count: ".to_string()), "count: 3".to_string());
                    }
                    _ => panic!(),
                }
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_vec_fn_builder() {
        // We can't directly compare closures, so just make sure we thread
        // through the builder.

        let mut count = 0u;
        let data = VecBuilder::new()
            .push_fn(|s| {
                count += 1u;
                s + count.to_string()
            })
            .build();

        match data {
            Data::Vec(vs) => {
                match &vs {
                    [Data::Fun(ref f)] => {
                        let f = &mut *f.borrow_mut();
                        assert_eq!((*f)("count: ".to_string()), "count: 1".to_string());
                        assert_eq!((*f)("count: ".to_string()), "count: 2".to_string());
                        assert_eq!((*f)("count: ".to_string()), "count: 3".to_string());
                    }
                    _ => panic!(),
                }
            }
            _ => panic!(),
        }
    }
}
