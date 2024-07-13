(module
  ;; Import our myprint function
  (import "myenv" "print" (func $print (param i64 i32)))

  ;; Define a single page memory of 64KB.
  (memory $0 1)

  ;; Store the Hello World (null terminated) string at byte offset 0
  (data (i32.const 100) "Hello World!\n")

  (func $printHello
    i64.const 100
    i32.const 13
    (call $print)
  )

  (func (export "_start")
    (call $printHello)
  )
)