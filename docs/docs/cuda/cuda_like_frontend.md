---
sidebar_position: 1
---

# Develop, Optimize, and Deploy your Expander-Accelerated zkApp

The Polyhedra's zkCUDA provides a development enviroment for creating high-performance GKR native circuits. With zkCUDA, you can leverage the power of Expander GKR prover and hardware's parallelism to accelerate your proof generation without losing the expressiveness of GKR circuits. The language should be familiar to CUDA users, with similar syntax and semantics. And it's written in Rust.

With zkCUDA, you can:

1. Easily develop high-performance GKR native circuits.
2. Easily leverage the power of distributed hardware and highly parallel hardware like GPU or MPI-enabled clusters.

## Programming Model

Due to the nature of GKR circuits, the programming model is very similar to CUDA. We define following concepts:

1. zk Streaming Multiprocessors (zkSMs): Each zkSM is a parallel instance of the same circuit, it can run up to 16 zk threads in parallel, one or more zkSMs can be executed in parallel.
2. zk Threads: A zk thread is the smallest unit of execution, it's executed in a zkSM where the thread's execution is independent of other threads in the same zkSM.
3. Device Memory: the device memory is the memory that can be directly accessed by zkSM.
4. Host Memory: the host memory is the memory that can be directly accessed by the your own CPU.
5. Kernel: a kernel is a function that describes the circuit computation.

## Basic Concepts mapping from CUDA to Crytpography

1. Memory Copy:
    a. from host to device: this operation will copy data from host memory to device memory. It's done by using polynomial commitment, each memory copy will generate a commitment to the data array.

    b. from device to host: this operation will copy data from device memory to host memory. It's done by opening the polynomial commitment (either single or batched) and output the polynomial evaluation to the host.

2. zkSM: It's a concept that describes log-space uniformity of the circuit.

3. Kernel function: it will be a circuit described by a variant of gnark frontend in Rust.

4. Lookup tables: a lookup table is a powerful tool to optimize the circuit, we will provide a built in dict type for users to use it flexbilely. You need to fill the dict with data in host, then perform the host to device memory copy to send the data to device.

## Programming Guides [WIP]

It's still working in progress, not usable now, and the APIs may change in the future. Please raise issue in github if you have any questions or suggestions.

## 1. Kernel Function Definition

The kernel function in this CUDA-like circuit frontend is similar to a CUDA kernel. It aligns with the current type of memorize call functions. This function will run under Zero-Knowledge (ZK) conditions, with the proof automatically maintained by contexts.

```rust
fn example_kernel<C: Config>(api: &mut API<C>, inputs: &[&[Variable]]) -> Vec<Variable> {
    vec![api.mul(inputs[0][0], inputs[0][1])]
}
```

Functions with more complex parameters might look like the following:

```rust
#[kernel]
fn complex_kernel<C: Config>(
    api: &mut API<C>,
    inputs: &[&[Variable]]
) -> Vec<Variable> {
    // Implementation
}
```

## 2. Context

The context automatically maintains the existing proof and commits the input variables. It provides a series of functions as follows:

```rust
fn init_ctx<C: Config>() -> Context<C> {
    // Implementation
}

impl<C: Config> Context<C> {
    fn copy_from_host<T: IntoFlattenedFieldAndShape<C>>(&mut self, vars: T) -> DeviceMemory<C> {
        // Implementation
    }

    fn copy_to_host<T: FromFlattenedFieldAndShape<C>>(&self, dev_mem: &DeviceMemory<C>) -> T {
        // Implementation
    }

    fn call_kernel<F>(&mut self, f: F, inputs: &[DeviceMemory<C>]) -> Result<DeviceMemory<C>, KernelError>
    where
        F: Fn(&mut API<C>, &[&[Variable]]) -> Vec<Variable>,
    {
        // Implementation
    }

    fn get_proof(&mut self) -> Proof {
        // Implementation
    }
}
```

## 3. DeviceMemory

The `DeviceMemory<C>` struct represents memory on the device (in this case, the ZK circuit). It has the following internal structure:

```rust
struct DeviceMemory<C: Config> {
    values: Vec<C::CircuitField>,
    shape: Vec<usize>,
    // Additional internal fields for device-specific usage
}

impl<C: Config> DeviceMemory<C> {
    fn reshape(&mut self, new_shape: Vec<usize>) -> Result<(), ReshapeError> {
        // Implementation
    }

    fn flatten(&mut self) {
        // Implementation
    }

    fn dim(&self) -> &[usize] {
        &self.shape
    }

    fn parallelize(&self) -> Vec<DeviceMemory<C>> {
        // Split DeviceMemory based on the first dimension
        // Return a Vec<DeviceMemory<C>>, where each DeviceMemory represents a parallel instance
    }
}
```

Note that `DeviceMemory` does not expose a public `new` method, as instances should only be created through the `Context`.

## 4. IntoFlattenedFieldAndShape and FromFlattenedFieldAndShape Traits

To support copying nested arrays and vectors of arbitrary dimensions to and from the device, we define the following traits:

```rust
trait IntoFlattenedFieldAndShape<C: Config> {
    fn into_flattened_field_and_shape(self) -> (Vec<C::CircuitField>, Vec<usize>);
}

trait FromFlattenedFieldAndShape<C: Config>: Sized {
    fn from_flattened_field_and_shape(values: Vec<C::CircuitField>, shape: Vec<usize>) -> Self;
}
```

These traits can be implemented for various nested structures of `Vec` and arrays.

## 5. Kernel API (ExpanderCompilerCollection)

The Kernel API, also known as ExpanderCompilerCollection (ECC), provides a builder API similar to gnark. It includes the following operations:

```rust
pub trait BasicAPI<C: Config> {
    fn add(&mut self, x: impl ToVariableOrValue<C::CircuitField>, y: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn sub(&mut self, x: impl ToVariableOrValue<C::CircuitField>, y: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn mul(&mut self, x: impl ToVariableOrValue<C::CircuitField>, y: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn div(&mut self, x: impl ToVariableOrValue<C::CircuitField>, y: impl ToVariableOrValue<C::CircuitField>, checked: bool) -> Variable;
    fn neg(&mut self, x: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn inverse(&mut self, x: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn is_zero(&mut self, x: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn xor(&mut self, x: impl ToVariableOrValue<C::CircuitField>, y: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn or(&mut self, x: impl ToVariableOrValue<C::CircuitField>, y: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn and(&mut self, x: impl ToVariableOrValue<C::CircuitField>, y: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn assert_is_zero(&mut self, x: impl ToVariableOrValue<C::CircuitField>);
    fn assert_is_non_zero(&mut self, x: impl ToVariableOrValue<C::CircuitField>);
    fn assert_is_bool(&mut self, x: impl ToVariableOrValue<C::CircuitField>);
    fn assert_is_equal(&mut self, x: impl ToVariableOrValue<C::CircuitField>, y: impl ToVariableOrValue<C::CircuitField>);
    fn assert_is_different(&mut self, x: impl ToVariableOrValue<C::CircuitField>, y: impl ToVariableOrValue<C::CircuitField>);
}
```

## 6. Kernel Execution

The `call_kernel` method handles the parallelization of kernel execution:

```rust
fn call_kernel<F>(&mut self, f: F, inputs: &[DeviceMemory<C>]) -> Result<DeviceMemory<C>, KernelError>
where
    F: Fn(&mut API<C>, &[&[Variable]]) -> Vec<Variable>,
{
    // Check if the first dimension is consistent across all inputs
    let parallel_count = inputs[0].shape[0];
    if !inputs.iter().all(|dm| dm.shape[0] == parallel_count) {
        return Err(KernelError::InconsistentParallelCount);
    }

    // Parallelize all inputs
    let parallelized_inputs: Vec<Vec<DeviceMemory<C>>> = inputs
        .iter()
        .map(|dm| dm.parallelize())
        .collect();

    // Execute parallel kernel calls
    let mut results = Vec::with_capacity(parallel_count);
    for i in 0..parallel_count {
        // Implementation
    }

    // Merge results
    let merged_results = self.merge_results(results);
    Ok(merged_results)
}
```

## 7. Example Usage

Here's an example of how to use this CUDA-like circuit frontend:

```rust
fn kernel_func<C: Config>(api: &mut API<C>, inputs: &[&[Variable]]) -> Vec<Variable> {
    let a = inputs[0];
    let b = inputs[1];
    let sum = api.add(a[0], a[1]);
    vec![sum]
}

fn main() {
    let mut ctx = init_ctx();
    // Create 3 parallel instances, each with 2 elements
    let a = ctx.copy_from_host(vec![vec![1u32, 2u32], vec![3u32, 4u32], vec![5u32, 6u32]]);
    // Create 3 parallel instances, each with 1 element
    let b = ctx.copy_from_host(vec![3u32, 7u32, 11u32]);
    
    let result = ctx.call_kernel(kernel_func, &[a, b]).unwrap();
    
    // result's shape will be [3, 1], representing 3 parallel instances, each outputting 1 result
    assert_eq!(result.shape, vec![3, 1]);
    
    let proof = ctx.get_proof();
}
```