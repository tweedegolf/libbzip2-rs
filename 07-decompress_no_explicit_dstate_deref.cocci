@@
@@

-unsafe fn makeMaps_d(s: *mut DState) {
+unsafe fn makeMaps_d(s: &mut DState) {
...
}

@@
@@

-pub unsafe fn BZ2_decompress(s: *mut DState) -> i32 {
+pub unsafe fn BZ2_decompress(s: &mut DState) -> i32 {
...
}

@@
@@

- (*s)
+ s
