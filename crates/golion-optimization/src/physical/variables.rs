/// Physical Variables stores definition.
/// It includes also their creation trait.
use golion_domain::asset::bess::limits::BessLimits;
use good_lp::{ProblemVariables, Variable, variable};

pub struct BessVariables {
    // We are using Box instead of Vec here since variables
    // are fixed in size.
    pub input_power: Box<[Variable]>,
    pub output_power: Box<[Variable]>,
    pub soc: Box<[Variable]>,
}

pub trait VariableCreation<T> {
    fn create_variables(&self, variable_store: &mut ProblemVariables, length: usize)
    -> T;
}
impl VariableCreation<BessVariables> for BessLimits {
    fn create_variables(
        &self,
        variable_store: &mut ProblemVariables,
        length: usize,
    ) -> BessVariables {
        let input_power: Box<[Variable]> = (0..length)
            .map(|_| variable_store.add(variable().min(0.0).max(self.max_input_power)))
            .collect();
        let output_power: Box<[Variable]> = (0..length)
            .map(|_| variable_store.add(variable().min(0.0).max(self.max_output_power)))
            .collect();
        let soc: Box<[Variable]> = (0..length)
            .map(|_| variable_store.add(variable().min(self.min_soc).max(self.max_soc)))
            .collect();
        BessVariables { input_power, output_power, soc }
    }
}
