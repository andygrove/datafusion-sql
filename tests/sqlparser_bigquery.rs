// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![warn(clippy::all)]
//! Test SQL syntax specific to Google BigQuery. The parser based on the
//! generic dialect is also tested (on the inputs it can handle).

use sqlparser::ast::*;
use sqlparser::dialect::BigQueryDialect;
use sqlparser::test_utils::*;

#[test]
fn parse_bigquery() {
    let query = bq()
        .verified_query("SELECT name FROM `bigquery-public-data`.usa_names.usa_1910_2013 LIMIT 10");
    assert_eq!(Expr::Value(number("10")), query.limit.unwrap());
}

#[test]
fn parse_simple_select() {
    let sql =  "SELECT name, SUM(number) AS total_people FROM `bigquery-public-data`.usa_names.usa_1910_2013 LIMIT 20";
    let select = bq().verified_only_select(sql);
    assert_eq!(
        &Expr::Identifier(Ident::new("name")),
        expr_from_projection(&select.projection[0])
    );

    let sum_func = ObjectName(vec![Ident::new("SUM")]);
    let args = vec![Expr::Identifier(Ident::new("number"))];
    let alias = Ident::new("total_people");
    assert_eq!(
        &SelectItem::ExprWithAlias {
            expr: Expr::Function(Function {
                name: sum_func,
                args: args,
                over: None,
                distinct: false,
            }),
            alias: alias,
        },
        &select.projection[1],
    );

    let qualifiers = vec![
        Ident::with_quote('`', "bigquery-public-data"), // GCP project
        Ident::new("usa_names"),                        // dataset
        Ident::new("usa_1910_2013"),                    // table
    ];
    match &select.from[0].relation {
        TableFactor::Table {
            name,
            alias: _,
            args: _,
            with_hints: _,
        } => assert_eq!(&ObjectName(qualifiers), name),
        _ => panic!("not a table name"),
    };
}

#[test]
fn parse_timestamp() {
    let query =
        "SELECT a FROM t WHERE _time BETWEEN TIMESTAMP('2019-07-15') AND TIMESTAMP('2019-07-30')";
    let select = bq().verified_only_select(query);
}

fn bq() -> TestedDialects {
    TestedDialects {
        dialects: vec![Box::new(BigQueryDialect {})],
    }
}
