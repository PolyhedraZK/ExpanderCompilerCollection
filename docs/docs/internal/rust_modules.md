---
sidebar_position: 4
---

# Rust Internal Modules

The following modules are available:

## builder
The main purpose of this module is to process the raw intermediate representation (IR), primarily consisting of add/mul operations, into a quad expression representation. It is similar to the builder in `gnark` or the pure Go ExpanderCompilerCollection. This module is used in various compilation steps.

## circuit
Defines various circuits, including IR and the final layered circuit.

### ir
Defines several types of IR. During the compilation process, the code is first directly converted to the source IR and then transformed multiple times to obtain the destination IR. Some common code is defined in the `common` module.

### layered
Defines the layered circuit. This is the final compilation product that can be used by Expander.

### Others
- `config`: Defines compilation parameters corresponding to various fields.
- `costs`: Defines parameters for optimization during the compilation process.
- `input_mapping`: Defines the mapping of inputs. During the compilation process, unnecessary inputs may be removed. In such cases, the original inputs need to be mapped to the new inputs.

## compile
Contains the main logic for compilation. It sequentially calls various parts to compile the source IR into a layered circuit.

## hints
Currently defines some built-in hints. Hints, similar to those in `gnark`, are non-add/mul computations introduced during the witness solving process.

## layering
Contains the logic for converting the destination IR into a layered circuit. The architecture of this part is consistent with what is described in [this document](./compilation_process).

## utils
Various utility functions.
