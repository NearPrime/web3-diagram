use crate::{
    md_api::MdAPI,
    objects::{
        connection::{self, Connection, ConnectionType},
        node::{ActionType, Node, ScopeType},
    },
    syntax::{flow_chart::FlowChart, FlowDirection},
};
use scanner_syn::contract_descriptor::{
    ContractDescriptor, ContractInfo, DefaultContractDescriptor, FunctionInfo,
};
use std::{
    ops::{Deref, DerefMut},
    vec::Vec,
};
struct Connections(Vec<Connection>);
impl Deref for Connections {
    type Target = Vec<Connection>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Connections {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Into<ScopeType> for FunctionInfo {
    fn into(self) -> ScopeType {
        if self.is_public {
            ScopeType::Public
        } else if !self.is_public {
            ScopeType::Private
        } else if self.is_trait_impl {
            ScopeType::Trait
        } else if self.is_payable {
            ScopeType::Payable
        } else {
            ScopeType::Public
        }
    }
}
impl Into<ActionType> for FunctionInfo {
    fn into(self) -> ActionType {
        if self.is_event {
            ActionType::Event
        } else if self.is_mutable {
            ActionType::Mutation
        } else if self.is_process {
            ActionType::Process
        } else if self.is_view {
            ActionType::View
        } else {
            ActionType::None
        }
    }
}
impl Into<ConnectionType> for FunctionInfo {
    fn into(self) -> ConnectionType {
        if self.is_event {
            ConnectionType::Emission
        } else if self.is_trait_impl {
            ConnectionType::CrossContractConnection
        } else {
            ConnectionType::DirectConnection
        }
    }
}

impl From<Vec<FunctionInfo>> for Connections {
    fn from(val: Vec<FunctionInfo>) -> Self {
        if !val.is_empty() {
            let inner = val
                .into_iter()
                .map(|ifn| -> Connection {
                    Connection {
                        connection_type: ifn.clone().into(),
                        node: Node {
                            name: ifn.name.clone(),
                            scope: ifn.clone().into(),
                            action: ifn.clone().into(),
                            connections: Connections::from(ifn.clone().inner_calls.unwrap()).0,
                        },
                    }
                })
                .collect();

            return Connections(inner);
        }
        Connections(Vec::new())
    }
}

pub struct ScannerPipeline {
    content: String,
}
impl ScannerPipeline {
    fn from(contract: ContractInfo, flow_direction: FlowDirection) -> ScannerPipeline {
        let mut hierarchy_tree_root = Node {
            name: "Contract".to_string(),
            scope: ScopeType::Contract,
            action: ActionType::None,
            connections: Vec::new(),
        };
        contract
            .contract_metadata
            .into_iter()
            .enumerate()
            .for_each(|(_, value)| {
                hierarchy_tree_root
                    .connections
                    .extend(Connections::from(value.fns).0);
            });

        let mut api = MdAPI::<FlowChart>::new(flow_direction, hierarchy_tree_root);
        let result = api.parse_hierarchy();

        ScannerPipeline { content: result }
    }
}