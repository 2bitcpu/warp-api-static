$(function(){
    let jqXHR;
    $("#hello").on("click",function(){
        $.ajax({url:"/service/manage/other/hello"}).done(function(data){
            console.log("done");
            console.log(data);
        });
    });
    $("#wait").on("click",function(){
        if(jqXHR){
            console.log(jqXHR.readyState);
            return;
        }
        jqXHR = $.ajax({url:"/service/manage/other/wait",timeout:5000})
        .done(function(data){
            console.log("done");
            console.log(data);
        })
        .fail(function(){
            console.log("fail");
        });
    });
});