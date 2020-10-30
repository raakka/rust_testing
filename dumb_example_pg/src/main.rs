///////////////////////////////////////////////////////////////////////////////////////////////////
// Config Struct 

mod config {
	pub use ::config::ConfigError;
	use serde::Deserialize;
	#[derive(Deserialize)]

	pub struct Config {
		pub server_addr: String,
		pub pg: deadpool_postgres::Config,
	}

	impl Config {
		pub fn from_env() -> Result<Self, ConfigError> {
			let mut cfg = ::config::Config::new();
			cfg.merge(::config::Environment::new())?;
			cfg.try_into()
		}
	}
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// Mapping/Migrations

mod models {
	use serde::{Deserialize, Serialize};
	use tokio_pg_mapper_derive::PostgresMapper;
	
	#[derive(Deserialize, PostgresMapper, Serialize)]
	#[pg_mapper(table = "akamai")]
	pub struct Akamai {
		pub _abck: String,
		pub site: String,
	}
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// Custom Error Handling

mod errors [
	use acrtix_web::{HttpResponse, ResponseError};
	use deadpool_postgres::PoolError;
	use derive_more::{Display, From};
	use tokio_pg_mapper::Error as PGMError;
	
	#[derive(Display, From, Debug)]
	pub enum MyError {
		NotFound,
		PGError(PGError),
		PGMError(PGMError),
		PoolError(PoolError),
	}
	impl std::error::Error for MyError {}
	
	impl ResponseError for MyError {
		fn error_response(&self) -> HttpResponse {
			match *self {
				MyError::NotFound => HttpResponse::NotFound().finish(),
				MyError::PoolError(ref err) => {
					HttpResponse::InternalServerError().body(err.to_string())
				}
				_ => HttpResponse::InternalServerError().finish,
			}
		}
	}	
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// Queries

mod db {
	use crate::{errors::MyError, models::Akamai};
	use deadpool_postgres::Client;
	use tokio_pg_mapper::FromTokioPostgresRow;
	
	pub async fn get_cookie(client: &Client, akamai_info: Akamai) -> Result<Akamai, MyError> {
		let _stmt = include_str!("../sql/querycookie.sql");
		let _stmt = _stmt.replace("table_fields", &Akamai::sql_table_fields());
		let stmt = client.prepare(&_stmt).await.unwrap();

	client
		.query(
			&stmt,
			&[
				&_abck,
				&site,
			],
		)
		.await?
		.iter()
		.map(|row| Akamai::from_row_ref(row).unwrap())
		.collect::<Vec<Akamai>>()
		.pop()
		.ok_or(MyError::NotFound)
	}
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// Handlers

mod handlers {
    use crate::{ db, errors::MyError, models::Akamai };
    use actix_web::{ web, Error, HttpResponse };
    use deadpool_postgres::{ Client, Pool };

    pub async fn serve_cookie (
        cookie: web::Json<Akamai>,
        db_deadpool: web::Data<Pool>,
    ) -> Result<HttpResponse, Error> {
        let cookie_inf: cookie.into_inner();
        let client: Client = db_deadpool.get().await.map_err(MyError::PoolError)?;
        let generated_cookie = db::get_cookie(&client, cookie_inf).await?;
        Ok(HttpResponse::Ok().json(generated_cookie)
    }
}
