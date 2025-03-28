use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::{Formatter, Debug};
use std::sync::Arc;

/*
    This small, dumb and inefficient solution will solve problems of storing values in bytesware's.    
    This solution is highly inspired by axum Extension type, but im too dumb to implement it in the right way.

    Example:
    ```
        pub fn main() {
        let mut cool = Varmap {
            map: HashMap::new()
        };

        cool.insert::<&str>("world");
        let new_cool = cool.clone();
        println!("Map 1: {:?}\nMap 2: {:?}", cool, new_cool);
        println!("Value 1: {:?}\nValue 2: {:?}", cool.get::<&str>(), new_cool.get::<&str>());
        cool.insert::<&str>("not world");
        println!("Map 1: {:?}\nMap 2: {:?}", cool, new_cool);
        println!("Value 1: {:?}\nValue 2: {:?}", cool.get::<&str>(), new_cool.get::<&str>());
    }
    ```
*/

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


#[allow(dead_code)]
impl Varmap {
    pub fn new() -> Self {
        Varmap {
            map: HashMap::new()
        }
    }

    pub fn insert<T: Any + Send + Sync>(&mut self, value: T) {
        let type_id = TypeId::of::<T>();
        self.map.insert(type_id, Arc::new(Box::new(value)));
    }

    pub fn get<T: Any + Send + Sync>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.map.get(&type_id)?.downcast_ref::<T>()
    }

    pub fn remove<T: Any + Send + Sync>(&mut self) {
        let type_id = TypeId::of::<T>();
        self.map.remove(&type_id);
    }
}
