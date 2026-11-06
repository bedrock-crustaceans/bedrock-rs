//! `tiers` bodies: a villager's trade table.
//!
//! Layout, every count one byte and every integer an `i32`:
//!
//! ```text
//! tiers:  count × { groups: count × group, total_exp_required }
//! group:  num_to_select, trades: count × trade
//! trade:  wants: count × choice, gives: count × choice,
//!         trader_exp, max_uses, 01, weight
//! choice: count × item
//! item:   quantity min, quantity max, item (string), functions: count (always 0),
//!         02 00 00 00 00 00 00 00 00, price_multiplier (f32)
//! ```
//!
//! A member the JSON twin leaves out is written with its default: quantity 1 to 1,
//! `price_multiplier` 0.0, `trader_exp` 1, `max_uses` 7, `weight` 1. The text has a member
//! only when it is not at that default, as the twins do. The byte after `max_uses` (01 in
//! every trade) and the nine bytes after `functions` are fields no vanilla table varies; a
//! table with a function is not baked (vanilla ships those as JSON).
// Checked against the 22 distinct trade tables of the game's packs, each equal to its JSON twin.

use super::McbValue;
use super::reader::{Decoded, Members, Reader, UNSUPPORTED, object, string};

/// The nine bytes between an item's functions and its price multiplier.
const ITEM_PINNED: [u8; 9] = [2, 0, 0, 0, 0, 0, 0, 0, 0];

/// The payload of a trade table document: the `tiers` array.
pub(super) fn decode(bytes: &[u8]) -> Decoded<McbValue> {
    let reader = &mut Reader::new(bytes);
    let tiers = reader.list(tier)?;
    reader.end()?;
    Ok(tiers)
}

fn int(value: i32) -> McbValue {
    McbValue::Int(i64::from(value))
}

fn tier(reader: &mut Reader<'_>) -> Decoded<McbValue> {
    let groups = reader.list(group)?;
    let total_exp_required = reader.i32()?;
    Ok(object(vec![
        ("groups".into(), groups),
        ("total_exp_required".into(), int(total_exp_required)),
    ]))
}

fn group(reader: &mut Reader<'_>) -> Decoded<McbValue> {
    let num_to_select = reader.i32()?;
    let trades = reader.list(trade)?;
    Ok(object(vec![
        ("num_to_select".into(), int(num_to_select)),
        ("trades".into(), trades),
    ]))
}

fn trade(reader: &mut Reader<'_>) -> Decoded<McbValue> {
    let wants = reader.list(choice)?;
    let gives = reader.list(choice)?;
    let trader_exp = reader.i32()?;
    let max_uses = reader.i32()?;
    reader.pinned(&[1])?;
    let weight = reader.i32()?;
    let mut members: Members = vec![("wants".into(), wants), ("gives".into(), gives)];
    if trader_exp != 1 {
        members.push(("trader_exp".into(), int(trader_exp)));
    }
    if max_uses != 7 {
        members.push(("max_uses".into(), int(max_uses)));
    }
    if weight != 1 {
        members.push(("weight".into(), int(weight)));
    }
    Ok(object(members))
}

fn choice(reader: &mut Reader<'_>) -> Decoded<McbValue> {
    let items = reader.list(item)?;
    Ok(object(vec![("choice".into(), items)]))
}

fn item(reader: &mut Reader<'_>) -> Decoded<McbValue> {
    let min = reader.i32()?;
    let max = reader.i32()?;
    let name = reader.string()?;
    // A function's layout was never seen: a table that has one is not baked.
    if reader.count()? != 0 {
        return Err(UNSUPPORTED);
    }
    reader.pinned(&ITEM_PINNED)?;
    let price_multiplier = reader.f32()?;
    let mut members = Members::new();
    if (min, max) != (1, 1) {
        let mut quantity = Members::new();
        if min != 1 {
            quantity.push(("min".into(), int(min)));
        }
        quantity.push(("max".into(), int(max)));
        members.push(("quantity".into(), object(quantity)));
    }
    members.push(("item".into(), string(name)));
    members.push(("functions".into(), McbValue::Array(Vec::new())));
    if price_multiplier.to_bits() != 0 {
        members.push(("price_multiplier".into(), McbValue::F32(price_multiplier)));
    }
    Ok(object(members))
}
