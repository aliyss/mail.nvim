syntax clear
syntax case match


" TODO: Add support to directly match the table row values
syntax match mailFileIDRow      /ID\s*|\zs.\+\ze|/
syntax match mailFileFromRow    /From\s*|\zs.\+\ze|/
syntax match mailFileToRow      /To\s*|\zs.\+\ze|/
syntax match mailFileCCRow      /CC\s*|\zs.\+\ze|/
syntax match mailFileBCCRow     /BCC\s*|\zs.\+\ze|/
syntax match mailFileSubjectRow /Subject\s*|\zs.\+\ze|/
syntax match mailFileDateRow    /Date\s*|\zs.\+\ze|/

syntax match mailFileIDKey      /ID\ze\s*|/
syntax match mailFileFromKey    /From\ze\s*|/
syntax match mailFileToKey      /To\ze\s*|/
syntax match mailFileCCKey      /CC\ze\s*|/
syntax match mailFileBCCKey     /BCC\ze\s*|/
syntax match mailFileSubjectKey /Subject\ze\s*|/
syntax match mailFileDateKey    /Date\ze\s*|/


syntax match mailTableBorder /[+\-|]/
highlight default link mailTableBorder Delimiter

syn region mailTableHeader start="+++" end="+++" fold

setlocal foldmethod=syntax
setlocal foldlevel=0
setlocal foldcolumn=0
setlocal fillchars=fold:\ 

function! ClearFoldText()
    let l:line_count = v:foldend - v:foldstart + 1
    let l:text = "+++ Metadata (" . l:line_count . " lines) +++"
    return l:text
endfunction

setlocal foldtext=ClearFoldText()

hi! link mailFileIDRow      Identifier
hi! link mailFileFromRow    String
hi! link mailFileToRow      String
hi! link mailFileCCRow      String
hi! link mailFileBCCRow     String
hi! link mailFileSubjectRow Title
hi! link mailFileDateRow    Constant
hi! link mailFileIDKey      Identifier
hi! link mailFileFromKey    String
hi! link mailFileToKey      String
hi! link mailFileCCKey      String
hi! link mailFileBCCKey     String
hi! link mailFileSubjectKey Title
hi! link mailFileDateKey    Constant
hi! link Folded mailTableHeader

let b:current_syntax = "mail-table"


