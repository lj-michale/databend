// Copyright 2022 Datafuse Labs.
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

use common_datavalues::ToDataType;
use common_datavalues::VariantObjectType;
use common_datavalues::Vu8;
use common_expression::DataField;
use common_expression::DataSchemaRef;
use common_expression::DataSchemaRefExt;
use common_expression::SchemaDataType;
use common_meta_types::UserStageInfo;
use time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PresignAction {
    Download,
    Upload,
}

#[derive(Debug, Clone)]
pub struct PresignPlan {
    pub stage: Box<UserStageInfo>,
    pub path: String,
    pub action: PresignAction,
    pub expire: Duration,
}

impl PresignPlan {
    pub fn schema(&self) -> DataSchemaRef {
        DataSchemaRefExt::create(vec![
            DataField::new("method", SchemaDataType::String),
            DataField::new("headers", SchemaDataType::Variant),
            DataField::new("url", SchemaDataType::String),
        ])
    }
}
