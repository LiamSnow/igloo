// TODO glacier should register Floe #0 As internal
// We should be able to get:
//  - Current users name, uid, etc.
//  - Current time, date, datetime, weekday
// For things like Weather, etc they should be separate providers
// What should dummies be?

use std::time::Duration;
use tokio::time::sleep;

// mod auth;
mod glacier;
#[cfg(test)]
mod test;
// mod web;

// #[derive(Debug, Clone)]
// pub struct GlobalState {
//     query_tx: QueryEngineTx,
//     cast: broadcast::Sender<(u16, Message)>,
//     /// send msgs to dash tasks
//     dash_tx: broadcast::Sender<(u16, DashboardRequest)>,
//     dashs: Arc<RwLock<FxHashMap<String, Dashboard>>>,
// }

#[derive(Debug, Clone)]
pub enum DashboardRequest {
    Shutdown,
    DumpData,
}

#[tokio::main]
async fn main() {
    // let _auth = Auth::load().await.unwrap();

    let query_tx = glacier::spawn().await.unwrap();

    // wait for devices to connect
    sleep(Duration::from_secs(5)).await;

    // let (tx, mut rx) = mpsc::channel(10);

    // query_tx
    //     .send(QueryEngineRequest::Register(tx))
    //     .await
    //     .unwrap();

    // let QueryEngineResponse::Registered { producer_id } = rx.recv().await.unwrap() else {
    //     panic!()
    // };
    // println!("registered!");

    // let mut query = Query::Component(ComponentQuery {
    //     device_filter: DeviceFilter::default(),
    //     entity_filter: EntityFilter::default(),
    //     action: ComponentAction::GetValue,
    //     component: ComponentType::Dimmer,
    //     post_op: Some(AggregationOp::Mean),
    //     include_parents: true,
    //     limit: None,
    // });

    // let start = Instant::now();
    // let res = rx.recv().await.unwrap();
    // println!("got {res:?} in {:?}", start.elapsed());

    // let iterations = 10_000;

    // for _ in 0..iterations {
    //     query_tx.send((query.clone(), tx.clone())).await.unwrap();
    //     let _ = rx.recv().await.unwrap();
    // }

    // let elapsed = start.elapsed();
    // println!("Completed {} queries in {:?}", iterations, elapsed);
    // println!("Avg: {:?} per query", elapsed / iterations);
    // let query = Query {
    //     action: QueryAction::GetAggregate(igloo_interface::types::agg::AggregationOp::Mean),
    //     target: QueryTarget::Components(ComponentType::Dimmer),
    //     entity_filter: Some(EntityFilter::Has(ComponentType::Light)),
    //     ..Default::default()
    // };
    // query_tx.send((query.clone(), tx.clone())).await.unwrap();
    // let res = rx.recv().await.unwrap();
    // println!("{res:#?}");

    // let num_steps = 200;
    // let start = Instant::now();

    // for i in 0..num_steps {
    //     let brightness = (i as f64 / num_steps as f64) * 0.5; // 0.0 to 0.5

    //     let query = Query {
    //         action: QueryAction::Set(IglooValue::Real(brightness)),
    //         target: QueryTarget::Components(ComponentType::Dimmer),
    //         entity_filter: Some(EntityFilter::Has(ComponentType::Light)),
    //         ..Default::default()
    //     };

    //     query_tx.send((query, tx.clone())).await.unwrap();
    //     let res = rx.recv().await.unwrap();
    //     assert!(res.is_ok());

    //     tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    // }

    // let elapsed = start.elapsed();
    // println!(
    //     "{} queries in {:?} ({:.2?} per query, {:.0} qps)",
    //     num_steps,
    //     elapsed,
    //     elapsed / num_steps,
    //     num_steps as f64 / elapsed.as_secs_f64()
    // );

    // let start = Instant::now();
    // for _ in 0..1000 {
    //     query_tx.send((query.clone(), tx.clone())).await.unwrap();
    //     rx.recv().await;
    // }
    // println!(
    //     "1000 queries: {:?} ({:?} per query)",
    //     start.elapsed(),
    //     start.elapsed() / 1000
    // );

    // let gs = GlobalState {
    //     query_tx,
    //     cast: broadcast::channel(100).0,
    //     dash_tx: broadcast::channel(10).0,
    //     dashs: Arc::new(RwLock::new(FxHashMap::default())),
    // };

    // let gsc = gs.clone();
    // tokio::spawn(async move {
    //     web::run(gsc).await.unwrap();
    // });

    // tokio::select! {
    //     _ = tokio::signal::ctrl_c() => {
    //         println!("Shutting down Igloo");
    //     }
    // }
}
