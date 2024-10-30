@@
@@

-pub unsafe fn BZ2_blockSort(s: *mut EState) {
+pub unsafe fn BZ2_blockSort(s: &mut EState) {
...
}

@@
@@

- (*s)
+ s
