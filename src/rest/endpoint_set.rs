use std::sync::Arc;

use super::endpoint::Endpoint;
use super::partial_request::PartialRequest;
use super::Request;
use crate::blueprint::Blueprint;
use crate::http::RequestContext;
use crate::merge_right::MergeRight;
use crate::rest::operation::OperationQuery;
use crate::runtime::TargetRuntime;
use crate::valid::Validator;

/// Collection of endpoints
#[derive(Default, Clone, Debug)]
pub struct EndpointSet<Status> {
    endpoints: Vec<Endpoint>,
    marker: std::marker::PhantomData<Status>,
}

/// Represents a validated set of endpoints
#[derive(Default, Clone, Debug)]
pub struct Checked;

/// Represents a set of endpoints that haven't been validated yet.
#[derive(Default, Clone, Debug)]
pub struct Unchecked;

impl From<Endpoint> for EndpointSet<Unchecked> {
    fn from(endpoint: Endpoint) -> Self {
        let mut set = EndpointSet::default();
        set.add_endpoint(endpoint);
        set
    }
}

impl EndpointSet<Unchecked> {
    pub fn add_endpoint(&mut self, endpoint: Endpoint) {
        self.endpoints.push(endpoint);
    }

    pub fn try_new(operations: &str) -> anyhow::Result<EndpointSet<Unchecked>> {
        let mut set = EndpointSet::default();

        for endpoint in Endpoint::try_new(operations)? {
            set.add_endpoint(endpoint);
        }

        Ok(set)
    }

    pub fn extend(&mut self, other: EndpointSet<Unchecked>) {
        self.endpoints.extend(other.endpoints);
    }

    pub async fn into_checked(
        self,
        blueprint: &Blueprint,
        target_runtime: TargetRuntime,
    ) -> anyhow::Result<EndpointSet<Checked>> {
        let mut operations = vec![];

        let req_ctx = RequestContext::new(target_runtime);
        let req_ctx = Arc::new(req_ctx);

        for endpoint in self.endpoints.iter() {
            let req = endpoint.clone().into_request();
            let operation_qry = OperationQuery::new(req, req_ctx.clone())?;
            operations.push(operation_qry);
        }
        super::operation::validate_operations(blueprint, operations)
            .await
            .to_result()?;
        Ok(EndpointSet {
            marker: std::marker::PhantomData::<Checked>,
            endpoints: self.endpoints,
        })
    }
}

impl MergeRight for EndpointSet<Unchecked> {
    fn merge_right(mut self, other: Self) -> Self {
        self.extend(other);
        self
    }
}

impl EndpointSet<Checked> {
    pub fn matches(&self, request: &Request) -> Option<PartialRequest> {
        self.endpoints.iter().find_map(|e| e.matches(request))
    }
}
