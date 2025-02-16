use crate::api::conn::Method;
use crate::api::conn::Param;
use crate::api::conn::Router;
use crate::api::opt::Range;
use crate::api::opt::Resource;
use crate::api::Connection;
use crate::api::Result;
use crate::method::Live;
use crate::sql::Id;
use crate::sql::Value;
use serde::de::DeserializeOwned;
use std::future::Future;
use std::future::IntoFuture;
use std::marker::PhantomData;
use std::pin::Pin;

/// A select future
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Select<'r, C: Connection, R> {
	pub(super) router: Result<&'r Router<C>>,
	pub(super) resource: Result<Resource>,
	pub(super) range: Option<Range<Id>>,
	pub(super) response_type: PhantomData<R>,
}

macro_rules! into_future {
	($method:ident) => {
		fn into_future(self) -> Self::IntoFuture {
			let Select {
				router,
				resource,
				range,
				..
			} = self;
			Box::pin(async move {
				let param = match range {
					Some(range) => resource?.with_range(range)?.into(),
					None => resource?.into(),
				};
				let mut conn = Client::new(Method::Select);
				conn.$method(router?, Param::new(vec![param])).await
			})
		}
	};
}

impl<'r, Client> IntoFuture for Select<'r, Client, Value>
where
	Client: Connection,
{
	type Output = Result<Value>;
	type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + Sync + 'r>>;

	into_future! {execute_value}
}

impl<'r, Client, R> IntoFuture for Select<'r, Client, Option<R>>
where
	Client: Connection,
	R: DeserializeOwned,
{
	type Output = Result<Option<R>>;
	type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + Sync + 'r>>;

	into_future! {execute_opt}
}

impl<'r, Client, R> IntoFuture for Select<'r, Client, Vec<R>>
where
	Client: Connection,
	R: DeserializeOwned,
{
	type Output = Result<Vec<R>>;
	type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + Sync + 'r>>;

	into_future! {execute_vec}
}

impl<'r, C> Select<'r, C, Value>
where
	C: Connection,
{
	/// Restricts the records selected to those in the specified range
	pub fn range(mut self, bounds: impl Into<Range<Id>>) -> Self {
		self.range = Some(bounds.into());
		self
	}
}

impl<'r, C, R> Select<'r, C, Vec<R>>
where
	C: Connection,
{
	/// Restricts the records selected to those in the specified range
	pub fn range(mut self, bounds: impl Into<Range<Id>>) -> Self {
		self.range = Some(bounds.into());
		self
	}
}

impl<'r, C, R> Select<'r, C, R>
where
	C: Connection,
	R: DeserializeOwned,
{
	/// Turns a normal select query into a live query
	///
	/// # Examples
	///
	/// ```no_run
	/// # use futures::StreamExt;
	/// # use surrealdb::opt::Resource;
	/// # use surrealdb::Result;
	/// # use surrealdb::Notification;
	/// # #[derive(Debug, serde::Deserialize)]
	/// # struct Person;
	/// #
	/// # #[tokio::main]
	/// # async fn main() -> surrealdb::Result<()> {
	/// # let db = surrealdb::engine::any::connect("mem://").await?;
	/// #
	/// // Select the namespace/database to use
	/// db.use_ns("namespace").use_db("database").await?;
	///
	/// // Listen to all updates on a table
	/// let mut stream = db.select("person").live().await?;
	/// # let _: Option<Result<Notification<Person>>> = stream.next().await;
	///
	/// // Listen to updates on a range of records
	/// let mut stream = db.select("person").range("jane".."john").live().await?;
	/// # let _: Option<Result<Notification<Person>>> = stream.next().await;
	///
	/// // Listen to updates on a specific record
	/// let mut stream = db.select(("person", "h5wxrf2ewk8xjxosxtyc")).live().await?;
	///
	/// // The returned stream implements `futures::Stream` so we can
	/// // use it with `futures::StreamExt`, for example.
	/// while let Some(result) = stream.next().await {
	///     handle(result);
	/// }
	///
	/// // Handle the result of the live query notification
	/// fn handle(result: Result<Notification<Person>>) {
	///     match result {
	///         Ok(notification) => println!("{notification:?}"),
	///         Err(error) => eprintln!("{error}"),
	///     }
	/// }
	/// #
	/// # Ok(())
	/// # }
	/// ```
	pub fn live(self) -> Live<'r, C, R> {
		Live {
			router: self.router,
			resource: self.resource,
			range: self.range,
			response_type: self.response_type,
		}
	}
}
