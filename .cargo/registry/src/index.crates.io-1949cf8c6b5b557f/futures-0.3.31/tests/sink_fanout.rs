use futures::{
    channel::mpsc,
    executor::block_on,
    future::join3,
    sink::SinkExt,
    stream::{
        self,
        StreamExt,
    },
};

#[test]
fn it_works() {
    let (tx1, rx1) = mpsc::channel(1);
    let (tx2, rx2) = mpsc::channel(2);
    let tx = tx1.fanout(tx2).sink_map_err(|_| ());

    let src = stream::iter((0..10).map(Ok));
    let fwd = src.forward(tx);

    let collect_fut1 = rx1.collect::<Vec<_>>();
    let collect_fut2 = rx2.collect::<Vec<_>>();
    let (_, vec1, vec2) = block_on(join3(fwd, collect_fut1, collect_fut2));

    let expected = (0..10).collect::<Vec<_>>();

    assert_eq!(vec1, expected);
    assert_eq!(vec2, expected);
}
