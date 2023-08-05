use serde::{Serialize, Deserialize};

/// A node in the graph
#[derive(Serialize)]
pub struct Node<M: Serialize> {
    /// The id of the node
    pub id: i32,
    /// The input vector of the node
    pub input: Vec<i32>,
    /// The output vector of the node
    pub output: Vec<i32>,
    /// The actor of the node
    pub actor: Box<dyn SerializableAct<M>>
}

#[derive(Serialize, Deserialize)]
enum Serializers {
    Toml,
    Json
}

pub trait Act<M>
{
    fn act(&self, msg: M) -> M;
}

pub trait SerializableAct<M: Serialize>: erased_serde::Serialize + Act<M> {}

erased_serde::serialize_trait_object!(<T> SerializableAct<T> where T: Serialize);

#[derive(Serialize, Deserialize)]
struct Debug {
    ser: Serializers
}

impl<M: Serialize> Act<M> for Debug 
    {
    fn act(&self, msg: M) -> M {
        let mut toml = String::new();
        let serializer = toml::Serializer::new(&mut toml);
        let _ = serde::Serialize::serialize(&msg, serializer);
        println!("{}", toml);
        msg
    }
}

mod test {
    use super::*;
    
    #[test]
    fn test_debug() {
        let a = Debug { ser: Serializers::Toml };
        let json = serde_json::json!({"a": {"b": ["test", "test"]}});
        let json2 = a.act(json.clone());
        assert_eq!(json, json2);
    }
}
