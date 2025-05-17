; ModuleID = '_test2.smolpp'
source_filename = "_test2.smolpp"
target triple = "arm64-apple-darwin24.4.0"

%dynamic_type_struct = type { i8, i64 }

@__smolpp_g_none_string = constant [5 x i8] c"None\00"
@__smolpp_g_true_string = constant [5 x i8] c"True\00"
@__smolpp_g_false_string = constant [6 x i8] c"False\00"
@__smolpp_g_int_fmt_string = constant [3 x i8] c"%d\00"
@__smolpp_g_int_fmt_string_newline = constant [4 x i8] c"%d\0A\00"
@__smolpp_g_panic_invalid_type_fmt_string = constant [39 x i8] c"PANIC: Invalid internal type value %d\0A\00"
@__smolpp_g_panic_unimplemented = constant [33 x i8] c"PANIC: LLVM not implemented yet\0A\00"
@__smolpp_g_error_type = constant [15 x i8] c"TypeError: %s\0A\00"

define i32 @main() {
entry:
  %alloca_var_a = alloca %dynamic_type_struct, align 8
  store %dynamic_type_struct { i8 4, i64 0 }, ptr %alloca_var_a, align 4
  %load_dest_value = load %dynamic_type_struct, ptr %alloca_var_a, align 4
  %type_field = extractvalue %dynamic_type_struct %load_dest_value, 0
  %assert_type_assign = and i8 %type_field, 4
  %convert_to_bool = icmp ne i8 0, %assert_type_assign
  br i1 %convert_to_bool, label %ok, label %panic

ok:                                               ; preds = %entry
  store %dynamic_type_struct { i8 4, i64 1 }, ptr %alloca_var_a, align 4
  %load_a = load %dynamic_type_struct, ptr %alloca_var_a, align 4
  %value_field = extractvalue %dynamic_type_struct %load_a, 1
  %printf_int = call i32 (ptr, ...) @printf(ptr @__smolpp_g_int_fmt_string_newline, i64 %value_field)
  ret i32 0

panic:                                            ; preds = %entry
  %panic_printf = call i32 (ptr, ...) @printf(ptr @__smolpp_g_error_type, [38 x i8] c"Incompatible types during assignation\00")
  call void @llvm.trap()
  unreachable
}

declare i32 @puts(ptr)

declare i32 @printf(ptr, ...)

; Function Attrs: cold noreturn nounwind
declare void @llvm.trap() #0

define void @__smolpp_f_generic_print(%dynamic_type_struct %0) {
__smolpp_f_generic_print_entry:
  %type_field = extractvalue %dynamic_type_struct %0, 0
  switch i8 %type_field, label %default [
    i8 1, label %case_none
    i8 2, label %case_bool
    i8 4, label %case_int
    i8 8, label %case_string
    i8 16, label %case_list
  ]

case_none:                                        ; preds = %__smolpp_f_generic_print_entry
  %puts_none = call i32 @puts(ptr @__smolpp_g_none_string)
  ret void

case_bool:                                        ; preds = %__smolpp_f_generic_print_entry
  %value_field = extractvalue %dynamic_type_struct %0, 1
  %value_as_bool = trunc i64 %value_field to i1
  %cmp = icmp ne i1 %value_as_bool, false
  br i1 %cmp, label %then, label %else

case_int:                                         ; preds = %__smolpp_f_generic_print_entry
  %value_field1 = extractvalue %dynamic_type_struct %0, 1
  %printf_int = call i32 (ptr, ...) @printf(ptr @__smolpp_g_int_fmt_string_newline, i64 %value_field1)
  ret void

case_string:                                      ; preds = %__smolpp_f_generic_print_entry
  %value_field2 = extractvalue %dynamic_type_struct %0, 1
  %str_ptr = inttoptr i64 %value_field2 to ptr
  %puts_string = call i32 @puts(ptr %str_ptr)
  ret void

case_list:                                        ; preds = %__smolpp_f_generic_print_entry
  %panic_printf = call i32 (ptr, ...) @printf(ptr @__smolpp_g_panic_unimplemented)
  call void @llvm.trap()
  ret void

default:                                          ; preds = %__smolpp_f_generic_print_entry
  %panic_printf3 = call i32 (ptr, ...) @printf(ptr @__smolpp_g_panic_invalid_type_fmt_string, i8 %type_field)
  call void @llvm.trap()
  unreachable

then:                                             ; preds = %case_bool
  %puts_bool_true = call i32 @puts(ptr @__smolpp_g_true_string)
  br label %end

else:                                             ; preds = %case_bool
  %puts_bool_false = call i32 @puts(ptr @__smolpp_g_false_string)
  br label %end

end:                                              ; preds = %else, %then
  ret void
}

attributes #0 = { cold noreturn nounwind }
