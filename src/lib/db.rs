use std::env;

use sqlx::{Column, Row};
use sqlx_postgres::{PgPool, PgPoolOptions};
use tokio::io::AsyncWriteExt;

use crate::lib::pgtype;


pub struct Db {
    pool: PgPool,
}

impl Db {
    pub async fn new() -> Db {
        Db{
            pool: init_db().await,
        }
    }


    // perform query and return (result_columns, result_rows) (converted to strings)
    pub async fn query(&mut self, query: &str) -> (Vec<String>, Vec<Vec<String>>) {
        // perform the query
        let result = sqlx::query(query)
                .fetch_all(&self.pool)
                .await
                .expect("failed to execute query");
         
        
        let mut result_strs: Vec<Vec<String>> = Vec::new();
        let mut result_cols = Vec::<String>::new();

        // use the column names from the first row as result_columns
        if result.len() > 0 {
            result_cols = result[0].columns().iter().map(| col | {
                    col.name().to_string()
            }).collect();
        }
    
        // push stringified rows into result_strs
        for row in result {
            result_strs.push(pgtype::stringify(&row));
        }
        
        (result_cols, result_strs)

    }

    pub async fn query_table_names(&mut self) -> Vec<String> {
        // get user defined table names and set them in app.tables
        let (_, table_names_row) = self.query(TABLES_QUERY).await;

        let mut table_names = Vec::<String>::new();
        table_names_row.iter().for_each(| item | {
            table_names.push(item[0].clone());
        });

        table_names
    }
}



fn get_db_url() -> String {
        env::var("DB_URL").expect("DB_URL must be set")
}


pub async fn init_db() -> PgPool {
    let db_url = get_db_url();
    let logval = format!("db_url = {:?}", db_url);
    tokio::io::stdout().write_all(logval.as_bytes()).await.expect("failed to write to stdout");
    tokio::io::stdout().flush().await.expect("failed to flush stdout");
    // std::io::stdout().flush().unwrap();
    // std::io::Stdout::flush().await;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(db_url.as_str())
        .await
        .expect("failed to connect to db");
    pool

}




pub static TABLES_QUERY: &str  = "SELECT table_name FROM information_schema.tables WHERE table_schema NOT IN ('pg_catalog', 'information_schema');";


