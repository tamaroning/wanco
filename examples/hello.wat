(module
  (type (;0;) (func (param i32 i32 i32 i32) (result i32)))

  ;; Import fd_write function
  (import "wasi_snapshot_preview1" "fd_write" (func $fd_write (type 0)))

  ;; Define a single page memory of 64KB.
  (memory $0 1)

  ;; Store the Hello World (null terminated) string at byte offset 16
  (data (i32.const 16) "Hello, World\n")

  (func $printHello
    (call $fd_write
            (i32.const 1) ;; file_descriptor - 1 for stdout
            (i32.const 0) ;; *iovs - The pointer to the iov array, which is stored at memory location 0
            (i32.const 1) ;; iovs_len - We're printing 1 string stored in an iov - so one.
            (i32.const 128) ;; nwritten - A place in memory to store the number of bytes written
    )
  )

  (func (export "_start")
    ;; iov_base
    (i32.store (i32.const 0) (i32.const 16)) 
    ;; iov_len
    (i32.store (i32.const 4) (i32.const 13))
    ;; Call the print function
    (call $printHello)
  )
)