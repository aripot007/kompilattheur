Compare list

Create an internal function `list_cmp(l1, l2)`
- `0` if every value `l1[i] == l2[i]`
- `-1` if first non equal value `l1[i] < l2[i]` or all value equal but `len(l1) < len(l2)`
- `1` if first non equal value `l1[i] > l2[i]` or all value equal but `len(l1) > len(l2)`
you will need to use `compare_int_values` in `comparison.rs`

For internal function:
1. create internal function in `comparison.rs`
2. add it in `internal_function.rs` file in enum
3. impl into
4. and call it in `init_internal_functions`

If user do `l1 < l2` do `list_cmp(l1, l2) == -1`
If user do `l1 > l2` do `list_cmp(l1, l2) == 1`
If user do `l1 == l2` do `list_cmp(l1, l2) == 0`
