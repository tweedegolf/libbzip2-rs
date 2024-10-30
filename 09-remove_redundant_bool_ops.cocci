@@
expression a;
@@

-a as libc::c_int as Bool
+a as Bool

@@
expression a;
@@

-if a as Bool == 0 {
+if a == 0 {
...
}

@@
expression a;
@@

-if a as Bool != 0 {
+if a != 0 {
...
}

@@
expression a;
@@

-if !(a as Bool != 0) {
+if a == 0 {
...
}

@@
@@

-if 1 == 0 {
+if false {
...
}
