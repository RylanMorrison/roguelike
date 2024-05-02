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

pub const ORC_CAMP: PrefabSection = PrefabSection{
    template : ORC_CAMP_RIGHT,
    width: 35,
    height: 27,
    placement: ( HorizontalPlacement::Right, VerticalPlacement::Center )
};

#[allow(dead_code)]
const ORC_CAMP_RIGHT: &str = "
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

pub const WARBOSS_DEN: PrefabSection = PrefabSection{
    template: WARBOSS_DEN_TEMPLATE,
    width: 53,
    height: 23,
    placement: (HorizontalPlacement::Center, VerticalPlacement::Top)
};

const WARBOSS_DEN_TEMPLATE: &str = "
#####################################################
#  #####  #####    ##     >     ##    #####  #####  #
#  #   #  #   #     ##         ##     #   #  #   #  #
#  ##+##  ##+##      #####+####       ##+##  ##+##  #
#                                                   #
##########################+##########################
#                                                   #
#                      o  W  o                      #
#                                                   #
#                                                   #
##########################+##########################
#                                                   #
#                       O   O                       #
#                                                   #
######################         ######################
#                                                   #
#                                                   #
#                                                   #
#                  g g g g g g g g                  #
#                                                   #
#                                                   #
#                                                   #
#####################          ######################
";
