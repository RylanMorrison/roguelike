#[derive(PartialEq, Copy, Clone)]
pub enum HorizontalPlacement { Left, Center, Right }

#[derive(PartialEq, Copy, Clone)]
pub enum VerticalPlacement { Top, Center, Bottom }

#[derive(PartialEq, Copy, Clone)]
pub struct PrefabSection {
    pub template : &'static str,
    pub width : usize,
    pub height: usize,
    pub placement : (HorizontalPlacement, VerticalPlacement)
}

pub const ORC_CAMP : PrefabSection = PrefabSection{
    template : ORC_CAMP_RIGHT,
    width: 35,
    height: 27,
    placement: ( HorizontalPlacement::Right, VerticalPlacement::Center )
};

#[allow(dead_code)]
const ORC_CAMP_RIGHT : &str = "
      #############################
      #  #####  #####  #####  #####
      #  # o #  # o #  # o #  # o #
      #  ## ##  ## ##  ## ##  ## ##
      #                            
      #                            
      #  #####  #####  #####  #####
      #  # o #  # o #  # o #  # o #
      #  ## ##  ## ##  ## ##  ## ##
   o  #                            
                                   
   o  #                            
   o  #                            
                                O >
   o  #                            
   o  #                            
                                   
   o  #                            
      #  ## ##  ## ##  ## ##  ## ##
      #  # o #  # o #  # o #  # o #
      #  #####  #####  #####  #####
      #                            
      #                            
      #  ## ##  ## ##  ## ##  ## ##
      #  # o #  # o #  # o #  # o #
      #  #####  #####  #####  #####
      #############################
";
