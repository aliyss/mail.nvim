syntax clear
syntax case match

" 1. JSON-like Header with Folding
" We keep mailTableHeader as the name for the region
syn region mailTableHeader start="+++" end="+++" fold

" 2. Table Borders (Unicode)
syn match mailTableBorder /[│┼─]/
syn match mailTableSeparator /-\{3,\}/

" 3. Values
syn match mailTableFlag /\\\w\+/
syn keyword mailTableBool Yes No
syn match mailTableID /^\s*\zs[^│]\+\ze│/

" 4. Linking
hi def link mailTableHeader    Comment
hi def link mailTableBorder    Delimiter
hi def link mailTableSeparator Delimiter
hi def link mailTableFlag      Special
hi def link mailTableBool      Boolean
hi def link mailTableID        Function

" 5. Buffer Behavior
setlocal foldmethod=syntax
setlocal foldlevel=0
setlocal foldcolumn=0
setlocal fillchars=fold:\ 

" 6. Improved Fold Text with Line Count
function! ClearFoldText()
    " Calculate the number of lines in the fold
    let l:line_count = v:foldend - v:foldstart + 1
    " Craft the display string: Label + line count
    let l:text = "+++ Metadata (" . l:line_count . " lines) +++"
    return l:text
endfunction

setlocal foldtext=ClearFoldText()

hi! link Folded mailTableHeader

let b:current_syntax = "mail-table"
