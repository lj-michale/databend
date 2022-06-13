// Copyright 2021 Datafuse Labs.
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

use std::str::FromStr;
use std::sync::Arc;

use async_stream::stream;
use common_exception::Result;
use common_planners::PlanNode;
use common_streams::SendableDataBlockStream;
use common_tracing::tracing;
use futures::StreamExt;
use poem::error::BadRequest;
use poem::error::InternalServerError;
use poem::error::Result as PoemResult;
use poem::post;
use poem::web::Query;
use poem::Body;
use poem::Endpoint;
use poem::EndpointExt;
use poem::Route;
use serde::Deserialize;

use crate::formats::output_format::OutputFormatType;
use crate::interpreters::InterpreterFactory;
use crate::pipelines::new::processors::port::OutputPort;
use crate::pipelines::new::processors::StreamSource;
use crate::pipelines::new::SourcePipeBuilder;
use crate::servers::http::v1::HttpQueryContext;
use crate::sessions::QueryContext;
use crate::sessions::SessionType;
use crate::sql::PlanParser;

#[derive(Deserialize)]
pub struct StatementHandlerParams {
    query: Option<String>,
}

async fn execute(
    ctx: Arc<QueryContext>,
    plan: PlanNode,
    format: Option<String>,
    input_stream: Option<SendableDataBlockStream>,
) -> Result<Body> {
    let interpreter = InterpreterFactory::get(ctx.clone(), plan.clone())?;
    let _ = interpreter
        .start()
        .await
        .map_err(|e| tracing::error!("interpreter.start.error: {:?}", e));
    let data_stream: SendableDataBlockStream =
        if ctx.get_settings().get_enable_new_processor_framework()? != 0
            && ctx.get_cluster().is_empty()
        {
            let output_port = OutputPort::create();
            let stream_source =
                StreamSource::create(ctx.clone(), input_stream, output_port.clone())?;
            let mut source_pipe_builder = SourcePipeBuilder::create();
            source_pipe_builder.add_source(output_port, stream_source);
            let _ = interpreter
                .set_source_pipe_builder(Option::from(source_pipe_builder))
                .map_err(|e| tracing::error!("interpreter.set_source_pipe_builder.error: {:?}", e));
            interpreter.execute(None).await?
        } else {
            interpreter.execute(input_stream).await?
        };
    let mut data_stream = ctx.try_create_abortable(data_stream)?;
    let format_setting = ctx.get_format_settings()?;
    let mut fmt = OutputFormatType::TSV;
    if let Some(format) = format {
        fmt = OutputFormatType::from_str(format.as_str())?;
    }

    let mut output_format = fmt.create_format(plan.schema());
    let header = output_format.serialize_prefix(&format_setting);
    let stream = stream! {
        yield header;
        while let Some(block) = data_stream.next().await {
            match block{
                Ok(block) => {
                    yield output_format.serialize_block(&block, &format_setting);
                },
                Err(err) => yield(Err(err)),
            };
        }
        yield output_format.finalize();
        let _ = interpreter
            .finish()
            .await
            .map_err(|e| tracing::error!("interpreter.finish error: {:?}", e));
    };

    Ok(Body::from_bytes_stream(stream))
}

#[poem::handler]
pub async fn clickhouse_handler_get(
    ctx: &HttpQueryContext,
    Query(params): Query<StatementHandlerParams>,
) -> PoemResult<Body> {
    let session = ctx.get_session(SessionType::ClickHouseHttpHandler);
    let context = session
        .create_query_context()
        .await
        .map_err(InternalServerError)?;

    let sql = params.query.unwrap_or_default();
    let (plan, format) = PlanParser::parse_with_format(context.clone(), &sql)
        .await
        .map_err(BadRequest)?;

    context.attach_query_str(&sql);
    execute(context, plan, format, None)
        .await
        .map_err(InternalServerError)
}

#[poem::handler]
pub async fn clickhouse_handler_post(
    ctx: &HttpQueryContext,
    body: Body,
    Query(params): Query<StatementHandlerParams>,
) -> PoemResult<Body> {
    let session = ctx.get_session(SessionType::ClickHouseHttpHandler);
    let ctx = session
        .create_query_context()
        .await
        .map_err(InternalServerError)?;

    let mut sql = params.query.unwrap_or_default();
    sql.push_str(body.into_string().await?.as_str());

    let (plan, format) = PlanParser::parse_with_format(ctx.clone(), &sql)
        .await
        .map_err(BadRequest)?;

    ctx.attach_query_str(&sql);
    execute(ctx, plan, format, None)
        .await
        .map_err(InternalServerError)
}

pub fn clickhouse_router() -> impl Endpoint {
    Route::new()
        .at(
            "/",
            post(clickhouse_handler_post).get(clickhouse_handler_get),
        )
        .with(poem::middleware::Compression)
}
