// Copyright 2021 Datafuse Labs
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::BTreeMap;

use common_catalog::plan::Filters;
use common_catalog::plan::Partitions;
use common_expression::FieldIndex;
use common_expression::RemoteExpr;
use common_meta_app::schema::CatalogInfo;
use common_meta_app::schema::TableInfo;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct UpdateSource {
    pub parts: Partitions,
    pub table_info: TableInfo,
    pub catalog_info: CatalogInfo,
    pub col_indices: Vec<usize>,
    pub query_row_id_col: bool,

    pub filters: Option<Filters>,
    pub update_list: Vec<(FieldIndex, RemoteExpr<String>)>,
    pub computed_list: BTreeMap<FieldIndex, RemoteExpr<String>>,
}
