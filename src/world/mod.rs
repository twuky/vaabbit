use pulz_arena::{Arena, Index};
use quadtree::QuadTree;
pub mod quadtree;

struct Actor {

}
struct Solid {

}
struct Zone {

}

enum WorldObject {
    Actor(Index),
    Solid(Index),
    Zone(Index)
}

struct World {
    actors: Arena<Actor>,
    solids: Arena<Solid>,
    zones: Arena<Zone>,

    tree: QuadTree<WorldObject>
}

