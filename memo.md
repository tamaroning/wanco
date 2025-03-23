
bb1:
    call func2()
    stackmap(...)
    br ret

ret:
    ret

この場合optimized outされる
- func2はtail callだから?