use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::{Formatter, Debug};
use std::sync::Arc;

/// ## Varmap
/// 
/// This struct is a Hashmap with `std::any::TypeId` as key and `Arc<Box<T>>`, where `T` is `dyn Any + Send + Sync`.
/// So you can add simple things like `Instant`  or `u32`, or even store custom `struct`!
/// 
/// ## Examples
/// 
/// ```
/// let varmap = Varmap::new();
/// 
/// varmap.insert("Cool message");
/// 
/// if let Some(val) = varmap.get::<&str>() {
///     println!("There was a message! {val}");
/// }
/// 
/// // If you want to be robust im removing values
/// varmap.remove::<&str>();
/// 
/// if let Some(val) = varmap.get::<&str>() {
///     // This example should not get value here, if does - create issue ticket, please :)
///     println!("There is still a message! {val}");
/// } else {
///     println!("Message was removed :(");
/// }
/// 
/// ```
/// 
/// ## Attention
/// 
/// If your custom struct fails to be added to the `Varmap`, try to implement `Debug` and `Clone` for it. 
pub struct Varmap {
    map: HashMap<TypeId, Arc<Box<dyn Any + Send + Sync>>>, // Problem with implementing clone for Box<dyn Any>? Just wrapped it in Arc<> and problem solved gg :)
}

impl Debug for Varmap {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> { 
        f.debug_struct("Varmap")
            .field("map", &self.map)
            .finish()
    }
}

/// I believe there is a better way to do this, but it works, and i have no complains...
impl Clone for Varmap {
    fn clone(&self) -> Self { 
        let mut new_map: HashMap<TypeId, Arc<Box<dyn Any + Send + Sync>>> = HashMap::new();

        for (key, value) in self.map.clone() {
            let type_id = key;
            let value = value.clone();
            new_map.insert(type_id, value);
        }

        Varmap { map: new_map }
    }
}


impl Varmap {
    /// Just creates new one, nothing too special
    pub fn new() -> Self {
        Varmap {
            map: HashMap::new()
        }
    }

    /// We simply take `TypeId` take it as a key, then we wrap incoming value into Arc<Box<>>.
    /// As simple as that. I hope its not that bad of the approach...
    /// 
    /// ## Examples
    /// 
    /// ```
    /// let varmap = Varmap::new();
    /// varmap.insert("Cool message");
    /// ```
    pub fn insert<T: Any + Send + Sync>(&mut self, value: T) {
        let type_id = TypeId::of::<T>();
        self.map.insert(type_id, Arc::new(Box::new(value)));
    }

    /// Function for retrieving data...
    /// 
    /// ## Examples
    /// 
    /// ```
    /// let varmap = Varmap::new();
    /// varmap.insert("Cool message");
    /// 
    /// if let Some(val) = varmap.get::<&str>() {
    ///     println!("There was a message! {:?}", val);
    /// }    
    /// ```
    pub fn get<T: Any + Send + Sync>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.map.get(&type_id)?.downcast_ref::<T>()
    }

    /// Function for removing data from the varmap. Idk why would you need it, but in any case
    /// 
    /// ## Example
    ///
    /// ```
    /// let varmap = Varmap::new();
    /// 
    /// varmap.insert("Cool message");
    /// 
    /// if let Some(val) = varmap.get::<&str>() {
    ///     println!("There was a message! {val}");
    /// }
    /// 
    /// // If you want to be robust im removing values
    /// varmap.remove::<&str>();
    /// 
    /// if let Some(val) = varmap.get::<&str>() {
    ///     // This example should not get value here, if does - create issue ticket, please :)
    ///     println!("There is still a message! {val}");
    /// } else {
    ///     println!("Message was removed :(");
    /// }
    /// 
    /// ```
    pub fn remove<T: Any + Send + Sync>(&mut self) {
        let type_id = TypeId::of::<T>();
        self.map.remove(&type_id);
    }
}
