#[derive(PartialEq, Copy, Clone)]
pub struct PrefabRoom {
    pub template : &'static str,
    pub width : usize,
    pub height: usize,
    pub first_depth: i32,
    pub last_depth: i32
}

pub const GUARDED_WEAPON: PrefabRoom = PrefabRoom{
    template: GUARDED_WEAPON_MAP,
    width: 5,
    height: 5,
    first_depth: 4,
    last_depth: 100
};

const GUARDED_WEAPON_MAP: &str = "
     
 o o 
  /  
 o o 
     
";

pub const GUARDED_SHIELD: PrefabRoom = PrefabRoom{
    template: GUARDED_SHIELD_MAP,
    width: 5,
    height: 5,
    first_depth: 4,
    last_depth: 100
};

const GUARDED_SHIELD_MAP: &str = "
     
 o o 
  0  
 o o 
     
";

pub const OGRE_TRIO: PrefabRoom = PrefabRoom{
    template: OGRE_TRIO_MAP,
    width: 10,
    height: 9,
    first_depth: 8,
    last_depth: 100
};

const OGRE_TRIO_MAP: &str = "
          
 ######## 
 # %  % # 
 #  O   # 
    O % # 
 #  O   # 
 # %  % # 
 ######## 
          
";

