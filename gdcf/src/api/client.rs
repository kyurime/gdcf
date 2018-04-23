use futures::Future;

use api::request::{LevelsRequest, LevelRequest};
use api::GDError;

use model::RawObject;


type One = Box<Future<Item=RawObject, Error=GDError> + 'static>;
type Many = Box<Future<Item=Vec<RawObject>, Error=GDError> + 'static>;

pub trait ApiClient {
    fn level(&self, req: &LevelRequest) -> One;
    fn levels(&self, req: &LevelsRequest) -> Many;

    fn spawn<F>(&self, f: F)
        where
            F: Future<Item=(), Error=()> + 'static;
}
