use crate::state::{ACCOUNTS, CONFIG, STATE, TEMPLATES};
use crate::ContractError;
use cosmwasm_std::{BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response, Uint64};
use warp_protocol::controller::template::{
    DeleteTemplateMsg, EditTemplateMsg, SubmitTemplateMsg, Template,
};
use warp_protocol::controller::State;

pub fn submit_template(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    data: SubmitTemplateMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if !info.funds.contains(&Coin {
        denom: "uluna".to_string(),
        amount: config.template_fee,
    }) {
        return Err(ContractError::TemplateFeeNotFound {});
    }

    if !ACCOUNTS().has(deps.storage, info.sender.clone()) {
        return Err(ContractError::AccountDoesNotExist {});
    }

    if data.name.len() > 140 {
        return Err(ContractError::NameTooLong {});
    }

    if data.name.is_empty() {
        return Err(ContractError::NameTooShort {});
    }

    if data.formatted_str.len() > 280 {
        return Err(ContractError::NameTooLong {});
    }

    if data.formatted_str.is_empty() {
        return Err(ContractError::NameTooShort {});
    }

    //todo: checks for vars based on string and msg

    let state = STATE.load(deps.storage)?;
    let msg_template = Template {
        id: state.current_template_id,
        owner: info.sender.clone(),
        name: data.name.clone(),
        kind: data.kind.clone(),
        msg: data.msg.clone(),
        formatted_str: data.formatted_str.clone(),
        vars: data.vars.clone(),
        condition: data.condition.clone(),
    };

    TEMPLATES.save(deps.storage, state.current_template_id.u64(), &msg_template)?;
    STATE.save(
        deps.storage,
        &State {
            current_job_id: state.current_job_id,
            current_template_id: state.current_template_id.saturating_add(Uint64::new(1)),
            q: state.q,
        },
    )?;

    let msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: config.fee_collector.to_string(),
        amount: vec![Coin::new((config.template_fee).u128(), "uluna")],
    });

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "submit_msg_template")
        .add_attribute("id", state.current_template_id)
        .add_attribute("owner", info.sender)
        .add_attribute("name", data.name)
        .add_attribute("kind", serde_json_wasm::to_string(&data.kind)?)
        .add_attribute("msg", data.msg)
        .add_attribute("formatted_str", data.formatted_str)
        .add_attribute("vars", serde_json_wasm::to_string(&data.vars)?))
}

pub fn edit_template(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    data: EditTemplateMsg,
) -> Result<Response, ContractError> {
    let template = TEMPLATES.load(deps.storage, data.id.u64())?;

    if info.sender != template.owner {
        return Err(ContractError::Unauthorized {});
    }

    if data.name.is_some() && data.clone().name.unwrap().len() > 140 {
        return Err(ContractError::NameTooLong {});
    }

    if data.name.is_some() && data.name.clone().unwrap().is_empty() {
        return Err(ContractError::NameTooShort {});
    }

    let t = TEMPLATES.update(deps.storage, data.id.u64(), |t| match t {
        None => Err(ContractError::TemplateDoesNotExist {}),
        Some(t) => Ok(Template {
            id: t.id,
            owner: t.owner,
            name: data.name.unwrap_or(t.name),
            kind: t.kind,
            msg: t.msg,
            formatted_str: t.formatted_str,
            vars: t.vars,
            condition: t.condition,
        }),
    })?;

    Ok(Response::new()
        .add_attribute("action", "submit_msg_template")
        .add_attribute("id", t.id)
        .add_attribute("owner", info.sender)
        .add_attribute("name", t.name)
        .add_attribute("kind", serde_json_wasm::to_string(&t.kind)?)
        .add_attribute("msg", t.msg)
        .add_attribute("formatted_str", t.formatted_str)
        .add_attribute("vars", serde_json_wasm::to_string(&t.vars)?))
}

pub fn delete_template(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    data: DeleteTemplateMsg,
) -> Result<Response, ContractError> {
    let template = TEMPLATES.load(deps.storage, data.id.u64())?;

    if info.sender != template.owner {
        return Err(ContractError::Unauthorized {});
    }

    TEMPLATES.remove(deps.storage, data.id.u64());

    Ok(Response::new()
        .add_attribute("action", "delete_template")
        .add_attribute("id", data.id))
}
