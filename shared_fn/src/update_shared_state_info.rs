use core_ui::state::PipelineState;
use shared_fn::write_debug_steps::write_step_cmd_debug;


pub fn state_updater_for_ui_good_display(step &'static str, line: &str, shared_state: &PipelineState) {
  if "Discover Nodes" = step {
    /*
  	normally here we would save the nodes discovered in a vec state and then when starting at the step of updates use the first one (controller)
  	and then would pop (delete that one as it is done at the end of the sequence) it and add it in another state that would store it as done like a `DoneState`
  	so that when we come back in the loop to this stage, we would still discover all nodes but the state would check if the name is in `DoneState`,
  	  if `yes` would pop it and use next available which is not in `DoneState` and the sequence would work based ont hat one (important as SSH needed for workers)
  	  if `no` we would work with that one.
  	  Therefore need to create two new states or one with two fields both being Vec<String>
  	  */
  	}
  } else if "Pull Repo Key" = step {
 
  } else if "Madison Version" = step  {
  	
  } else if "Upgrade Plan" = step {
  	
  } else if "Upgrade Apply" = step {
  	
  } else if "Upgrade Node" = step {
  	
  } else if "Veryfy Core DNS Proxy = step {
  	
  }	
}
