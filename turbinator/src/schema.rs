table! {
    answertable (contestnumber, answernumber) {
        contestnumber -> Int4,
        answernumber -> Int4,
        runanswer -> Varchar,
        yes -> Bool,
        fake -> Bool,
        updatetime -> Int4,
    }
}

table! {
    bkptable (contestnumber, sitenumber, bkpnumber) {
        contestnumber -> Int4,
        sitenumber -> Int4,
        bkpnumber -> Int4,
        usernumber -> Int4,
        bkpdate -> Int4,
        bkpfilename -> Varchar,
        bkpdata -> Oid,
        bkpstatus -> Varchar,
        bkpsize -> Int4,
        updatetime -> Int4,
    }
}

table! {
    clartable (contestnumber, clarsitenumber, clarnumber) {
        contestnumber -> Int4,
        clarsitenumber -> Int4,
        clarnumber -> Int4,
        usernumber -> Int4,
        clardate -> Int4,
        clardatediff -> Int4,
        clardatediffans -> Int4,
        clarproblem -> Int4,
        clardata -> Text,
        claranswer -> Nullable<Text>,
        clarstatus -> Varchar,
        clarjudge -> Nullable<Int4>,
        clarjudgesite -> Nullable<Int4>,
        updatetime -> Int4,
    }
}

table! {
    contesttable (contestnumber) {
        contestnumber -> Int4,
        contestname -> Varchar,
        conteststartdate -> Int4,
        contestduration -> Int4,
        contestlastmileanswer -> Nullable<Int4>,
        contestlastmilescore -> Nullable<Int4>,
        contestlocalsite -> Int4,
        contestpenalty -> Int4,
        contestmaxfilesize -> Int4,
        contestactive -> Bool,
        contestmainsite -> Int4,
        contestkeys -> Text,
        contestunlockkey -> Varchar,
        contestmainsiteurl -> Varchar,
        updatetime -> Int4,
    }
}

table! {
    langtable (contestnumber, langnumber) {
        contestnumber -> Int4,
        langnumber -> Int4,
        langname -> Varchar,
        langextension -> Varchar,
        updatetime -> Int4,
    }
}

table! {
    logtable (lognumber) {
        lognumber -> Int4,
        contestnumber -> Int4,
        sitenumber -> Int4,
        loguser -> Nullable<Int4>,
        logip -> Varchar,
        logdate -> Int4,
        logtype -> Varchar,
        logdata -> Text,
        logstatus -> Nullable<Varchar>,
    }
}

table! {
    problemtable (contestnumber, problemnumber) {
        contestnumber -> Int4,
        problemnumber -> Int4,
        problemname -> Varchar,
        problemfullname -> Nullable<Varchar>,
        problembasefilename -> Nullable<Varchar>,
        probleminputfilename -> Nullable<Varchar>,
        probleminputfile -> Nullable<Oid>,
        probleminputfilehash -> Nullable<Varchar>,
        fake -> Bool,
        problemcolorname -> Nullable<Varchar>,
        problemcolor -> Nullable<Varchar>,
        updatetime -> Int4,
    }
}

table! {
    runtable (contestnumber, runsitenumber, runnumber) {
        contestnumber -> Int4,
        runsitenumber -> Int4,
        runnumber -> Int4,
        usernumber -> Int4,
        rundate -> Int4,
        rundatediff -> Int4,
        rundatediffans -> Int4,
        runproblem -> Int4,
        runfilename -> Varchar,
        rundata -> Oid,
        runanswer -> Int4,
        runstatus -> Varchar,
        runjudge -> Nullable<Int4>,
        runjudgesite -> Nullable<Int4>,
        runanswer1 -> Int4,
        runjudge1 -> Nullable<Int4>,
        runjudgesite1 -> Nullable<Int4>,
        runanswer2 -> Int4,
        runjudge2 -> Nullable<Int4>,
        runjudgesite2 -> Nullable<Int4>,
        runlangnumber -> Int4,
        autoip -> Nullable<Varchar>,
        autobegindate -> Nullable<Int4>,
        autoenddate -> Nullable<Int4>,
        autoanswer -> Nullable<Text>,
        autostdout -> Nullable<Oid>,
        autostderr -> Nullable<Oid>,
        updatetime -> Int4,
    }
}

table! {
    sitetable (contestnumber, sitenumber) {
        contestnumber -> Int4,
        sitenumber -> Int4,
        siteip -> Varchar,
        sitename -> Varchar,
        siteactive -> Bool,
        sitepermitlogins -> Bool,
        sitelastmileanswer -> Nullable<Int4>,
        sitelastmilescore -> Nullable<Int4>,
        siteduration -> Nullable<Int4>,
        siteautoend -> Nullable<Bool>,
        sitejudging -> Nullable<Text>,
        sitetasking -> Nullable<Text>,
        siteglobalscore -> Varchar,
        sitescorelevel -> Int4,
        sitenextuser -> Int4,
        sitenextclar -> Int4,
        sitenextrun -> Int4,
        sitenexttask -> Int4,
        sitemaxtask -> Int4,
        updatetime -> Int4,
        sitechiefname -> Varchar,
        siteautojudge -> Nullable<Bool>,
        sitemaxruntime -> Int4,
        sitemaxjudgewaittime -> Int4,
    }
}

table! {
    sitetimetable (contestnumber, sitenumber, sitestartdate) {
        contestnumber -> Int4,
        sitenumber -> Int4,
        sitestartdate -> Int4,
        siteenddate -> Int4,
        updatetime -> Int4,
    }
}

table! {
    tasktable (contestnumber, sitenumber, tasknumber) {
        contestnumber -> Int4,
        sitenumber -> Int4,
        usernumber -> Int4,
        tasknumber -> Int4,
        taskstaffnumber -> Nullable<Int4>,
        taskstaffsite -> Nullable<Int4>,
        taskdate -> Int4,
        taskdatediff -> Int4,
        taskdatediffans -> Int4,
        taskdesc -> Nullable<Varchar>,
        taskfilename -> Nullable<Varchar>,
        taskdata -> Nullable<Oid>,
        tasksystem -> Bool,
        taskstatus -> Varchar,
        colorname -> Nullable<Varchar>,
        color -> Nullable<Varchar>,
        updatetime -> Int4,
    }
}

table! {
    usertable (contestnumber, usersitenumber, usernumber) {
        contestnumber -> Int4,
        usersitenumber -> Int4,
        usernumber -> Int4,
        username -> Varchar,
        userfullname -> Varchar,
        userdesc -> Nullable<Varchar>,
        usertype -> Varchar,
        userenabled -> Bool,
        usermultilogin -> Bool,
        userpassword -> Nullable<Varchar>,
        userip -> Nullable<Varchar>,
        userlastlogin -> Nullable<Int4>,
        usersession -> Nullable<Varchar>,
        usersessionextra -> Nullable<Varchar>,
        userlastlogout -> Nullable<Int4>,
        userpermitip -> Nullable<Varchar>,
        userinfo -> Nullable<Varchar>,
        updatetime -> Int4,
        usericpcid -> Nullable<Varchar>,
    }
}

allow_tables_to_appear_in_same_query!(
    answertable,
    bkptable,
    clartable,
    contesttable,
    langtable,
    logtable,
    problemtable,
    runtable,
    sitetable,
    sitetimetable,
    tasktable,
    usertable,
);
