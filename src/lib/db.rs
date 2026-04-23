use std::env;

use sqlx::{Column, Row};
use sqlx_postgres::{PgPool, PgPoolOptions, PgRow};
use crate::lib::pgtype;

static DB_URL_VAR_NAME: &str = "DB_URL";
static DB_URL_ERR: &str = "DB_URL must be set";
static DB_CONNECT_ERR: &str = "failed to connect to db";
static QUERY_FAILED: &str = "failed to execute query";
static TABLES_QUERY: &str  = "SELECT table_name FROM information_schema.tables WHERE table_schema NOT IN ('pg_catalog', 'information_schema');";


pub struct Db {
    pool: PgPool,
}

impl Db {
    pub async fn new() -> Db {
        
        let db_url = env::var(DB_URL_VAR_NAME).expect(DB_URL_ERR);
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(db_url.as_str())
            .await
            .expect(DB_CONNECT_ERR);
        Db{
            pool: pool,
        }
    }


    // perform query and return (result_columns, result_rows) (converted to strings)
    pub async fn query(&mut self, query: &str) -> (Vec<String>, Vec<Vec<String>>) {
        // perform the query
        let result = sqlx::query(query)
                .fetch_all(&self.pool)
                .await
                .expect(QUERY_FAILED);
         
        
        let mut result_strs: Vec<Vec<String>> = Vec::new();
        // use the column names from the first row as result_columns
        let result_cols = self.col_names(&result);
    
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

    pub fn col_names(&mut self, rows: &Vec<PgRow>) -> Vec<String> {
        let mut result_cols = Vec::<String>::new();
        if rows.len() > 0 {
            result_cols = rows[0].columns().iter().map(| col | {
                    col.name().to_string()
            }).collect();
        }
        result_cols
    }
}





