return {
    "aliyss/mail.nvim",
    build = "./INSTALL",
    cmd = "MailUI",
    config = function()
        require("mail_nvim")
    end,
}
