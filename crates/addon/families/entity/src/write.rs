//! Writing the newest struct: the payload in schema order, every kept unknown member after the
//! known ones at its level.

use bedrock_addon_core::document::{ComponentMap, ComponentRef, Extra};
use bedrock_addon_core::json::{EncodeError, Value, WriteStyle, write_json};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{BETA_VERSION, write_payload};
use facet_value::{VArray, VObject};
use indexmap::IndexMap;

use super::component::EntityComponent;
use super::{AnimateEntry, Description, Entity, Property, PropertyDefault, Scripts};

/// The whole document: `format_version` (`"beta"` for a beta document) and the payload.
pub(crate) fn document(
    entity: &Entity,
    version: FormatVersion,
    out: &mut Vec<u8>,
    style: WriteStyle,
) -> Result<(), EncodeError> {
    let written: Value = if version == BETA_VERSION {
        "beta".into()
    } else {
        version.to_string().into()
    };
    let mut root = VObject::new();
    root.insert("format_version", written);
    root.insert("minecraft:entity", payload(entity)?);
    write_json(&Value::from(root), out, style)
}

/// The payload object.
pub(crate) fn payload(entity: &Entity) -> Result<Value, EncodeError> {
    let mut object = VObject::new();
    object.insert("description", description(&entity.description));
    if entity.written.components || !entity.components.is_empty() {
        object.insert("components", components(&entity.components)?);
    }
    if entity.written.component_groups || !entity.component_groups.is_empty() {
        let mut groups = VObject::new();
        for (name, group) in &entity.component_groups {
            groups.insert(name.as_str(), components(&group.components)?);
        }
        object.insert("component_groups", groups);
    }
    if entity.written.events || !entity.events.is_empty() {
        let mut events = VObject::new();
        for event in entity.events.values() {
            events.insert(event.name(), event.node.clone());
        }
        object.insert("events", events);
    }
    if let Some(brain) = &entity.brain {
        object.insert("brain", write_payload(brain)?);
    }
    if !entity.upgrades.is_empty() {
        object.insert("upgrades", map(&entity.upgrades));
    }
    if let Some(deprecated) = &entity.deprecated {
        let mut side_car = VObject::new();
        if !deprecated.aliases.is_empty() {
            side_car.insert("aliases", map(&deprecated.aliases));
        }
        object.insert("__deprecated", side_car);
    }
    extend(&mut object, &entity.extra);
    Ok(object.into())
}

/// A map of JSON values as an object.
fn map(members: &IndexMap<String, Value>) -> Value {
    let mut object = VObject::new();
    for (key, value) in members {
        object.insert(key.as_str(), value.clone());
    }
    object.into()
}

/// Appends the kept unknown members.
fn extend(object: &mut VObject, extra: &Extra) {
    for (key, value) in extra.iter() {
        object.insert(key, value.clone());
    }
}

/// `description`.
fn description(description: &Description) -> Value {
    let mut object = VObject::new();
    object.insert("identifier", description.identifier.as_str());
    if let Some(runtime) = &description.runtime_identifier {
        object.insert("runtime_identifier", runtime.as_str());
    }
    if let Some(spawnable) = description.is_spawnable {
        object.insert("is_spawnable", spawnable);
    }
    if let Some(summonable) = description.is_summonable {
        object.insert("is_summonable", summonable);
    }
    if !description.properties.is_empty() {
        let mut properties = VObject::new();
        for (name, property) in &description.properties {
            properties.insert(name.as_str(), self::property(property));
        }
        object.insert("properties", properties);
    }
    if !description.animations.is_empty() {
        let mut animations = VObject::new();
        for (name, animation) in &description.animations {
            animations.insert(name.as_str(), animation.as_str());
        }
        object.insert("animations", animations);
    }
    if let Some(scripts) = &description.scripts {
        object.insert("scripts", self::scripts(scripts));
    }
    if let Some(category) = description.spawn_category {
        object.insert("spawn_category", category.as_str());
    }
    if let Some(mapping) = &description.property_memory_mapping {
        object.insert("property_memory_mapping", mapping.clone());
    }
    if let Some(version) = &description.entity_version {
        let mut upgrades = VArray::new();
        for entry in &version.upgrades {
            let mut item = VObject::new();
            item.insert("upgrade_to", entry.upgrade_to);
            item.insert("name", entry.name.as_str());
            upgrades.push(item);
        }
        let mut object_version = VObject::new();
        object_version.insert("upgrades", upgrades);
        object.insert("entity_version", object_version);
    }
    extend(&mut object, &description.extra);
    object.into()
}

/// One property.
fn property(property: &Property) -> Value {
    let mut object = VObject::new();
    object.insert("type", property.kind.as_str());
    if let Some(default) = &property.default {
        let value: Value = match default {
            PropertyDefault::Bool(b) => (*b).into(),
            PropertyDefault::Int(i) => (*i).into(),
            PropertyDefault::Float(f) => (*f).into(),
            PropertyDefault::Enum(s) => s.as_str().into(),
            PropertyDefault::Expression(source) => source.as_str().into(),
        };
        object.insert("default", value);
    }
    if let Some([min, max]) = &property.range {
        let mut range = VArray::new();
        range.push(min.clone());
        range.push(max.clone());
        object.insert("range", range);
    }
    if !property.values.is_empty() {
        let mut values = VArray::new();
        for value in &property.values {
            values.push(value.as_str());
        }
        object.insert("values", values);
    }
    if let Some(sync) = property.client_sync {
        object.insert("client_sync", sync);
    }
    extend(&mut object, &property.extra);
    object.into()
}

/// `scripts`.
fn scripts(scripts: &Scripts) -> Value {
    let mut object = VObject::new();
    let mut animate = VArray::new();
    for entry in &scripts.animate {
        match entry {
            AnimateEntry::Always(name) => animate.push(name.as_str()),
            AnimateEntry::Conditional(members) => {
                let mut member = VObject::new();
                for (name, condition) in members {
                    member.insert(name.as_str(), condition.as_str());
                }
                animate.push(member);
            }
        }
    }
    if scripts.written.animate || !animate.is_empty() {
        object.insert("animate", animate);
    }
    extend(&mut object, &scripts.extra);
    object.into()
}

/// A component map, in its order.
pub(crate) fn components(map: &ComponentMap<EntityComponent>) -> Result<Value, EncodeError> {
    let mut object = VObject::new();
    for entry in map.iter() {
        match entry {
            ComponentRef::Typed { name, value, .. } => {
                object.insert(name, component(value)?);
            }
            ComponentRef::Custom(custom) => {
                object.insert(custom.name.as_str(), custom.parameters.clone());
            }
            ComponentRef::Dynamic { name, json } | ComponentRef::Unknown { name, json } => {
                object.insert(name, json.clone());
            }
            _ => {}
        }
    }
    Ok(object.into())
}

/// One typed component as JSON.
#[allow(clippy::too_many_lines, reason = "one arm per component")]
pub fn component(value: &EntityComponent) -> Result<Value, EncodeError> {
    use EntityComponent as C;
    match value {
        C::Custom(custom) => Ok(custom.parameters.clone()),
        C::Unknown(json) | C::Dynamic(json) => Ok(json.clone()),
        // the components 1.26.60 types first.
        C::Buoyant(c) => write_payload(c),
        C::DashAction(c) => write_payload(c),
        C::MovementAmphibious(c) => write_payload(c),
        C::MovementBasic(c) => write_payload(c),
        C::MovementFly(c) => write_payload(c),
        C::MovementGeneric(c) => write_payload(c),
        C::MovementGlide(c) => write_payload(c),
        C::MovementHover(c) => write_payload(c),
        C::MovementJump(c) => write_payload(c),
        C::MovementSkip(c) => write_payload(c),
        C::MovementSway(c) => write_payload(c),
        C::RailMovement(c) => write_payload(c),
        C::Attack(c) => write_payload(c),
        C::Healable(c) => write_payload(c),
        C::SpellEffects(c) => write_payload(c),
        C::Strength(c) => write_payload(c),
        C::Teleport(c) => write_payload(c),
        C::Timer(c) => write_payload(c),
        C::Trail(c) => write_payload(c),
        C::Transformation(c) => write_payload(c),
        C::AnnotationBreakDoor(c) => write_payload(c),
        C::EconomyTradeTable(c) => write_payload(c),
        C::Inventory(c) => write_payload(c),
        C::PushableByBlock(c) => write_payload(c),
        C::Rideable(c) => write_payload(c),
        C::ScaleByAge(c) => write_payload(c),
        C::Shareables(c) => write_payload(c),
        C::Addrider(c) => write_payload(c),
        C::AdmireItem(c) => write_payload(c),
        C::AngerLevel(c) => write_payload(c),
        C::AnnotationOpenDoor(c) => write_payload(c),
        C::ApplyKnockbackRules(c) => write_payload(c),
        C::Barter(c) => write_payload(c),
        C::BehaviorAdmireItem(c) => write_payload(c),
        C::BehaviorAquaticChargeAttack(c) => write_payload(c),
        C::BehaviorAvoidBlock(c) => write_payload(c),
        C::BehaviorAvoidMobType(c) => write_payload(c),
        C::BehaviorBarter(c) => write_payload(c),
        C::BehaviorBeg(c) => write_payload(c),
        // the four goals typed at 1.26.60.
        C::BehaviorFloatTempt(c) => write_payload(c),
        C::BehaviorPanic(c) => write_payload(c),
        C::BehaviorSendEvent(c) => write_payload(c),
        C::BehaviorTempt(c) => write_payload(c),
        C::BehaviorBreakDoor(c) => write_payload(c),
        C::BehaviorBreed(c) => write_payload(c),
        C::BehaviorCelebrate(c) => write_payload(c),
        C::BehaviorCelebrateSurvive(c) => write_payload(c),
        C::BehaviorChargeAttack(c) => write_payload(c),
        C::BehaviorChargeHeldItem(c) => write_payload(c),
        C::BehaviorCircleAroundAnchor(c) => write_payload(c),
        C::BehaviorControlledByPlayer(c) => write_payload(c),
        C::BehaviorCroak(c) => write_payload(c),
        C::BehaviorDefendTrustedTarget(c) => write_payload(c),
        C::BehaviorDefendVillageTarget(c) => write_payload(c),
        C::BehaviorDelayedAttack(c) => write_payload(c),
        C::BehaviorDig(c) => write_payload(c),
        C::BehaviorDoorInteract(c) => write_payload(c),
        C::BehaviorDragonChargePlayer(c) => write_payload(c),
        C::BehaviorDragonDeath(c) => write_payload(c),
        C::BehaviorDragonFlaming(c) => write_payload(c),
        C::BehaviorDragonHoldingPattern(c) => write_payload(c),
        C::BehaviorDragonLanding(c) => write_payload(c),
        C::BehaviorDragonScanning(c) => write_payload(c),
        C::BehaviorDragonStrafePlayer(c) => write_payload(c),
        C::BehaviorDragonTakeoff(c) => write_payload(c),
        C::BehaviorDrinkMilk(c) => write_payload(c),
        C::BehaviorDrinkPotion(c) => write_payload(c),
        C::BehaviorDropItemFor(c) => write_payload(c),
        C::BehaviorEatBlock(c) => write_payload(c),
        C::BehaviorEatCarriedItem(c) => write_payload(c),
        C::BehaviorEatMob(c) => write_payload(c),
        C::BehaviorEmerge(c) => write_payload(c),
        C::BehaviorEquipItem(c) => write_payload(c),
        C::BehaviorExploreOutskirts(c) => write_payload(c),
        C::BehaviorFertilizeFarmBlock(c) => write_payload(c),
        C::BehaviorFindCover(c) => write_payload(c),
        C::BehaviorFindMount(c) => write_payload(c),
        C::BehaviorFindUnderwaterTreasure(c) => write_payload(c),
        C::BehaviorFireAtTarget(c) => write_payload(c),
        C::BehaviorFleeSun(c) => write_payload(c),
        C::BehaviorFloat(c) => write_payload(c),
        C::BehaviorFloatWander(c) => write_payload(c),
        C::BehaviorFollowCaravan(c) => write_payload(c),
        C::BehaviorFollowMob(c) => write_payload(c),
        C::BehaviorFollowOwner(c) => write_payload(c),
        C::BehaviorFollowParent(c) => write_payload(c),
        C::BehaviorFollowTargetLeader(c) => write_payload(c),
        C::BehaviorGoAndGiveItemsToNoteblock(c) => write_payload(c),
        C::BehaviorGoAndGiveItemsToOwner(c) => write_payload(c),
        C::BehaviorGoHome(c) => write_payload(c),
        C::BehaviorGuardianAttack(c) => write_payload(c),
        C::BehaviorHarvestFarmBlock(c) => write_payload(c),
        C::BehaviorHide(c) => write_payload(c),
        C::BehaviorHoldGround(c) => write_payload(c),
        C::BehaviorHover(c) => write_payload(c),
        C::BehaviorHurtByTarget(c) => write_payload(c),
        C::BehaviorInspectBookshelf(c) => write_payload(c),
        C::BehaviorInvestigateSuspiciousLocation(c) => write_payload(c),
        C::BehaviorJumpAroundTarget(c) => write_payload(c),
        C::BehaviorJumpToBlock(c) => write_payload(c),
        C::BehaviorKnockbackRoar(c) => write_payload(c),
        C::BehaviorLayDown(c) => write_payload(c),
        C::BehaviorLayEgg(c) => write_payload(c),
        C::BehaviorLeapAtTarget(c) => write_payload(c),
        C::BehaviorLookAtEntity(c) => write_payload(c),
        C::BehaviorLookAtPlayer(c) => write_payload(c),
        C::BehaviorLookAtTarget(c) => write_payload(c),
        C::BehaviorLookAtTradingPlayer(c) => write_payload(c),
        C::BehaviorMakeLove(c) => write_payload(c),
        C::BehaviorMeleeAttack(c) => write_payload(c),
        C::BehaviorMeleeBoxAttack(c) => write_payload(c),
        C::BehaviorMingle(c) => write_payload(c),
        C::BehaviorMountPathing(c) => write_payload(c),
        C::BehaviorMoveAroundTarget(c) => write_payload(c),
        C::BehaviorMoveIndoors(c) => write_payload(c),
        C::BehaviorMoveOutdoors(c) => write_payload(c),
        C::BehaviorMoveThroughVillage(c) => write_payload(c),
        C::BehaviorMoveToBlock(c) => write_payload(c),
        C::BehaviorMoveToLand(c) => write_payload(c),
        C::BehaviorMoveToLava(c) => write_payload(c),
        C::BehaviorMoveToLiquid(c) => write_payload(c),
        C::BehaviorMoveToPoi(c) => write_payload(c),
        C::BehaviorMoveToRandomBlock(c) => write_payload(c),
        C::BehaviorMoveToVillage(c) => write_payload(c),
        C::BehaviorMoveToWater(c) => write_payload(c),
        C::BehaviorMoveTowardsDwellingRestriction(c) => write_payload(c),
        C::BehaviorMoveTowardsHomeRestriction(c) => write_payload(c),
        C::BehaviorMoveTowardsTarget(c) => write_payload(c),
        C::BehaviorNap(c) => write_payload(c),
        C::BehaviorNearestAttackableTarget(c) => write_payload(c),
        C::BehaviorNearestPrioritizedAttackableTarget(c) => write_payload(c),
        C::BehaviorOcelotSitOnBlock(c) => write_payload(c),
        C::BehaviorOcelotattack(c) => write_payload(c),
        C::BehaviorOfferFlower(c) => write_payload(c),
        C::BehaviorOpenDoor(c) => write_payload(c),
        C::BehaviorOwnerHurtByTarget(c) => write_payload(c),
        C::BehaviorOwnerHurtTarget(c) => write_payload(c),
        C::BehaviorPetSleepWithOwner(c) => write_payload(c),
        C::BehaviorPickupItems(c) => write_payload(c),
        C::BehaviorPlaceBlock(c) => write_payload(c),
        C::BehaviorPlay(c) => write_payload(c),
        C::BehaviorPlayDead(c) => write_payload(c),
        C::BehaviorPlayerRideTamed(c) => write_payload(c),
        C::BehaviorRaidGarden(c) => write_payload(c),
        C::BehaviorRamAttack(c) => write_payload(c),
        C::BehaviorRandomBreach(c) => write_payload(c),
        C::BehaviorRandomFly(c) => write_payload(c),
        C::BehaviorRandomHover(c) => write_payload(c),
        C::BehaviorRandomLookAround(c) => write_payload(c),
        C::BehaviorRandomLookAroundAndSit(c) => write_payload(c),
        C::BehaviorRandomSearchAndDig(c) => write_payload(c),
        C::BehaviorRandomSitting(c) => write_payload(c),
        C::BehaviorRandomStroll(c) => write_payload(c),
        C::BehaviorRandomSwim(c) => write_payload(c),
        C::BehaviorRangedAttack(c) => write_payload(c),
        C::BehaviorReceiveLove(c) => write_payload(c),
        C::BehaviorRestrictOpenDoor(c) => write_payload(c),
        C::BehaviorRestrictSun(c) => write_payload(c),
        C::BehaviorRiseToLiquidLevel(c) => write_payload(c),
        C::BehaviorRoar(c) => write_payload(c),
        C::BehaviorRoll(c) => write_payload(c),
        C::BehaviorRunAroundLikeCrazy(c) => write_payload(c),
        C::BehaviorScared(c) => write_payload(c),
        C::BehaviorShareItems(c) => write_payload(c),
        C::BehaviorSilverfishMergeWithStone(c) => write_payload(c),
        C::BehaviorSilverfishWakeUpFriends(c) => write_payload(c),
        C::BehaviorSkeletonHorseTrap(c) => write_payload(c),
        C::BehaviorSleep(c) => write_payload(c),
        C::BehaviorSlimeAttack(c) => write_payload(c),
        C::BehaviorSlimeFloat(c) => write_payload(c),
        C::BehaviorSlimeKeepOnJumping(c) => write_payload(c),
        C::BehaviorSlimeRandomDirection(c) => write_payload(c),
        C::BehaviorSnacking(c) => write_payload(c),
        C::BehaviorSneeze(c) => write_payload(c),
        C::BehaviorSniff(c) => write_payload(c),
        C::BehaviorSonicBoom(c) => write_payload(c),
        C::BehaviorSquidDive(c) => write_payload(c),
        C::BehaviorSquidFlee(c) => write_payload(c),
        C::BehaviorSquidIdle(c) => write_payload(c),
        C::BehaviorSquidMoveAwayFromGround(c) => write_payload(c),
        C::BehaviorSquidOutOfWater(c) => write_payload(c),
        C::BehaviorStalkAndPounceOnTarget(c) => write_payload(c),
        C::BehaviorStayNearNoteblock(c) => write_payload(c),
        C::BehaviorStayWhileSitting(c) => write_payload(c),
        C::BehaviorStompAttack(c) => write_payload(c),
        C::BehaviorStompTurtleEgg(c) => write_payload(c),
        C::BehaviorStrollTowardsVillage(c) => write_payload(c),
        C::BehaviorSummonEntity(c) => write_payload(c),
        C::BehaviorSwell(c) => write_payload(c),
        C::BehaviorSwimIdle(c) => write_payload(c),
        C::BehaviorSwimUpForBreath(c) => write_payload(c),
        C::BehaviorSwimWander(c) => write_payload(c),
        C::BehaviorSwimWithEntity(c) => write_payload(c),
        C::BehaviorSwoopAttack(c) => write_payload(c),
        C::BehaviorTakeBlock(c) => write_payload(c),
        C::BehaviorTakeFlower(c) => write_payload(c),
        C::BehaviorTargetWhenPushed(c) => write_payload(c),
        C::BehaviorTeleportToOwner(c) => write_payload(c),
        C::BehaviorTimerFlag1(c) => write_payload(c),
        C::BehaviorTimerFlag2(c) => write_payload(c),
        C::BehaviorTimerFlag3(c) => write_payload(c),
        C::BehaviorTradeInterest(c) => write_payload(c),
        C::BehaviorTradeWithPlayer(c) => write_payload(c),
        C::BehaviorTransportItems(c) => write_payload(c),
        C::BehaviorUseKineticWeapon(c) => write_payload(c),
        C::BehaviorWitherRandomAttackPosGoal(c) => write_payload(c),
        C::BehaviorWitherTargetHighestDamage(c) => write_payload(c),
        C::BehaviorWork(c) => write_payload(c),
        C::BehaviorWorkComposter(c) => write_payload(c),
        C::BlockClimber(c) => write_payload(c),
        C::BlockMovementSlowdownImmunity(c) => write_payload(c),
        C::BodyRotationAlwaysFollowsHead(c) => write_payload(c),
        C::BodyRotationBlocked(c) => write_payload(c),
        C::Boss(c) => write_payload(c),
        C::Bucketable(c) => write_payload(c),
        C::BurnsInDaylight(c) => write_payload(c),
        C::CanClimb(c) => write_payload(c),
        C::CanFly(c) => write_payload(c),
        C::CanJoinRaid(c) => write_payload(c),
        C::CanPowerJump(c) => write_payload(c),
        C::CanStandOnPowderSnow(c) => write_payload(c),
        C::CannotBeAttacked(c) => write_payload(c),
        C::CollisionBox(c) => write_payload(c),
        C::DamageOverTime(c) => write_payload(c),
        C::DimensionBound(c) => write_payload(c),
        C::ExhaustionValues(c) => write_payload(c),
        C::ExperienceReward(c) => write_payload(c),
        C::FireImmune(c) => write_payload(c),
        C::FloatsInLiquid(c) => write_payload(c),
        C::Flocking(c) => write_payload(c),
        C::FreezingImmune(c) => write_payload(c),
        C::FreezingVulnerable(c) => write_payload(c),
        C::GameEventMovementTracking(c) => write_payload(c),
        C::GrowsCrop(c) => write_payload(c),
        C::Hide(c) => write_payload(c),
        C::Home(c) => write_payload(c),
        C::IgnoreCannotBeAttacked(c) => write_payload(c),
        C::InputGroundControlled(c) => write_payload(c),
        C::Insomnia(c) => write_payload(c),
        C::InstantDespawn(c) => write_payload(c),
        C::Interact(c) => write_payload(c),
        C::IsBaby(c) => write_payload(c),
        C::IsCharged(c) => write_payload(c),
        C::IsChested(c) => write_payload(c),
        C::IsHiddenWhenInvisible(c) => write_payload(c),
        C::IsIgnited(c) => write_payload(c),
        C::IsIllagerCaptain(c) => write_payload(c),
        C::IsPregnant(c) => write_payload(c),
        C::IsSaddled(c) => write_payload(c),
        C::IsShaking(c) => write_payload(c),
        C::IsSheared(c) => write_payload(c),
        C::IsStackable(c) => write_payload(c),
        C::IsStunned(c) => write_payload(c),
        C::IsTamed(c) => write_payload(c),
        C::ItemHopper(c) => write_payload(c),
        C::JumpDynamic(c) => write_payload(c),
        C::JumpStatic(c) => write_payload(c),
        C::ManagedWanderingTrader(c) => write_payload(c),
        C::MemoryBehaviorLookAtEntity(c) => write_payload(c),
        C::MemoryBehaviorMoveToPosition(c) => write_payload(c),
        C::MemoryBehaviorFollowTarget(c) => write_payload(c),
        C::MemorySensorInRangeOfEntity(c) => write_payload(c),
        // the 1.26.50 ram / jump Brain keys.
        C::MemoryBehaviorJumpToBlock(c) => write_payload(c),
        C::MemoryBehaviorPrepareRamAttack(c) => write_payload(c),
        C::MemoryBehaviorRamAttack(c) => write_payload(c),
        C::MemorySensorFindRamChargePosition(c) => write_payload(c),
        C::MemoryBehaviorWait(c) => write_payload(c),
        C::MemorySensorFindNearestBlock(c) => write_payload(c),
        C::MemorySensorFindNearestEntity(c) => write_payload(c),
        C::MemorySensorFindNearestPoi(c) => write_payload(c),
        C::MemorySensorGameEvent(c) => write_payload(c),
        C::NotPickableFromInside(c) => write_payload(c),
        C::OnDeath(c) => write_payload(c),
        C::OnEquipmentChanged(c) => write_payload(c),
        C::OnFriendlyAnger(c) => write_payload(c),
        C::OnHurt(c) => write_payload(c),
        C::OnHurtByPlayer(c) => write_payload(c),
        C::OnIgnite(c) => write_payload(c),
        C::OnStartLanding(c) => write_payload(c),
        C::OnStartTakeoff(c) => write_payload(c),
        C::OnTargetAcquired(c) => write_payload(c),
        C::OnTargetEscape(c) => write_payload(c),
        C::OnWakeWithOwner(c) => write_payload(c),
        C::OutOfControl(c) => write_payload(c),
        C::Persistent(c) => write_payload(c),
        C::Physics(c) => write_payload(c),
        C::PreferredPath(c) => write_payload(c),
        C::Projectile(c) => write_payload(c),
        C::PushableByEntity(c) => write_payload(c),
        C::RendersWhenInvisible(c) => write_payload(c),
        C::RotationAxisAligned(c) => write_payload(c),
        C::RotationLockedToVehicle(c) => write_payload(c),
        C::SpawnOnDeath(c) => write_payload(c),
        C::SuspectTracking(c) => write_payload(c),
        C::Tamemount(c) => write_payload(c),
        C::TickWorld(c) => write_payload(c),
        C::TradeResupply(c) => write_payload(c),
        C::TradeTable(c) => write_payload(c),
        C::Transient(c) => write_payload(c),
        C::Trust(c) => write_payload(c),
        C::UsesLegacyAmbientSounds(c) => write_payload(c),
        C::UsesLegacyFriction(c) => write_payload(c),
        C::UsesUniformAirDrag(c) => write_payload(c),
        C::VibrationDamper(c) => write_payload(c),
        C::VibrationListener(c) => write_payload(c),
        C::WantsJockey(c) => write_payload(c),
        C::WitherTargetHighestDamage(c) => write_payload(c),
    }
}
