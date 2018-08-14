use api::ApiClient;
use api::request::level::SearchFilters;
use api::request::LevelsRequest;
use api::response::ProcessedResponse;
use cache::Cache;
use error::CacheError;
use error::GdcfError;
use futures::Async;
use futures::Future;
use futures::future::Either;
use futures::future::join_all;
use model::GDObject;
use model::PartialLevel;
use std::error::Error;
use std::mem;
use std::sync::Arc;
use std::sync::Mutex;
use futures::future::result;

#[derive(Debug)]
struct Gdcf2<A: ApiClient + 'static, C: Cache + 'static> {
    client: Arc<Mutex<A>>,
    cache: Arc<Mutex<C>>,
}

impl<A: ApiClient + 'static, C: Cache + 'static> Clone for Gdcf2<A, C> {
    fn clone(&self) -> Self {
        Gdcf2 {
            client: self.client.clone(),
            cache: self.cache.clone()
        }
    }
}

// TODO: figure out the race conditions later

impl<A: ApiClient + 'static, C: Cache + 'static> Gdcf2<A, C> {
    pub fn levels(&self, req: LevelsRequest) -> GdcfFuture<Vec<PartialLevel>, A::Err, C::Err> {
        let cache = lock!(self.cache);
        let clone = self.clone();

        match cache.lookup_partial_levels(&req) {
            Ok(cached) => {
                if cache.is_expired(&cached) {
                    GdcfFuture::outdated(cached.extract(), clone.levels_future(req))
                } else {
                    GdcfFuture::up_to_date(cached.extract())
                }
            }

            Err(CacheError::CacheMiss) => GdcfFuture::absent(clone.levels_future(req)),

            Err(err) => panic!("Error accessing cache! {:?}", err)
        }
    }

    fn levels_future(self, req: LevelsRequest) -> impl Future<Item=Vec<PartialLevel>, Error=GdcfError<A::Err, C::Err>> + Send + 'static {
        let cache = self.cache.clone();
        let future = self.client.lock().unwrap().levels(&req);

        future.map_err(|api_error| GdcfError::Api(api_error))
            .and_then(move |response| self.integrity(response))
            .and_then(move |response| {
                let mut levels = Vec::new();
                let cache = cache.lock().unwrap();

                for obj in response {
                    match obj {
                        GDObject::PartialLevel(level) => levels.push(level),
                        _ => cache.store_object(&obj)?
                    }
                }

                cache.store_partial_levels(&req, &levels);
                Ok(levels)
            })
    }

    fn integrity(self, response: ProcessedResponse) -> impl Future<Item=ProcessedResponse, Error=GdcfError<A::Err, C::Err>> + Send + 'static {
        let mut reqs = Vec::new();

        for obj in &response {
            match obj {
                GDObject::Level(level) => {
                    if let Some(song_id) = level.base.custom_song_id {
                        match lock!(self.cache).lookup_song(song_id) {
                            Err(CacheError::CacheMiss) => {
                                reqs.push(self.levels(LevelsRequest::default()
                                    .with_id(level.base.level_id)
                                    .filter(SearchFilters::default()
                                        .custom_song(song_id)))
                                    .map(|_| ()))
                            }

                            Err(err) => {
                                return Either::B(result(Err(GdcfError::Cache(err))));
                            }

                            _ => continue
                        }
                    }
                }
                _ => ()
            }
        }

        if reqs.is_empty() {
            Either::B(result(Ok(response)))
        } else {
            Either::A(join_all(reqs)
                .map(move |_| response))
        }
    }
}

struct GdcfFuture<T, AE: Error + Send + 'static, CE: Error + Send + 'static> {
    // invariant: at least one of the fields is not `None`
    cached: Option<T>,
    refresher: Option<Box<dyn Future<Item=T, Error=GdcfError<AE, CE>> + Send + 'static>>,
}

impl<T, CE: Error + Send + 'static, AE: Error + Send + 'static> GdcfFuture<T, AE, CE> {
    fn up_to_date(object: T) -> GdcfFuture<T, AE, CE> {
        GdcfFuture {
            cached: Some(object),
            refresher: None,
        }
    }

    fn outdated<F>(object: T, f: F) -> GdcfFuture<T, AE, CE>
        where
            F: Future<Item=T, Error=GdcfError<AE, CE>> + Send + 'static
    {
        GdcfFuture {
            cached: Some(object),
            refresher: Some(Box::new(f)),
        }
    }

    fn absent<F>(f: F) -> GdcfFuture<T, AE, CE>
        where
            F: Future<Item=T, Error=GdcfError<AE, CE>> + Send + 'static
    {
        GdcfFuture {
            cached: None,
            refresher: Some(Box::new(f)),
        }
    }

    pub fn cached(&self) -> &Option<T> {
        &self.cached
    }

    pub fn take(&mut self) -> Option<T> {
        mem::replace(&mut self.cached, None)
    }
}

impl<T, AE: Error + Send + 'static, CE: Error + Send + 'static> Future for GdcfFuture<T, AE, CE> {
    type Item = T;
    type Error = GdcfError<AE, CE>;

    fn poll(&mut self) -> Result<Async<T>, GdcfError<AE, CE>> {
        match self.refresher {
            Some(ref mut fut) => fut.poll(),
            None => Ok(Async::Ready(self.take().unwrap()))
        }
    }
}