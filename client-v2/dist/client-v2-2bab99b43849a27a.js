let X=null,a2=`function`,a5=`string`,a3=`number`,a7=`Object`,Y=1,a0=0,aa=4,ae=266,_=`utf-8`,Z=`undefined`,af=609,a9=16,a4=`boolean`,V=Array,a6=Array.isArray,$=Error,a8=FinalizationRegistry,ac=Object,ad=Promise,ab=Reflect,a1=Uint8Array,W=undefined;var R=((a,b)=>{});var A=((b,c)=>{a._dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hae3eeb48d8770088(b,c)});var E=((b,c)=>{a._dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__he2299eea79ab0701(b,c)});var C=((b,c,d)=>{a._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h7925afe1ebb2315c(b,c,g(d))});var f=(a=>{const b=c(a);e(a);return b});var u=(a=>{const b=typeof a;if(b==a3||b==a4||a==X){return `${a}`};if(b==a5){return `"${a}"`};if(b==`symbol`){const b=a.description;if(b==X){return `Symbol`}else{return `Symbol(${b})`}};if(b==a2){const b=a.name;if(typeof b==a5&&b.length>a0){return `Function(${b})`}else{return `Function`}};if(a6(a)){const b=a.length;let c=`[`;if(b>a0){c+=u(a[a0])};for(let d=Y;d<b;d++){c+=`, `+ u(a[d])};c+=`]`;return c};const c=/\[object ([^\]]+)\]/.exec(toString.call(a));let d;if(c.length>Y){d=c[Y]}else{return toString.call(a)};if(d==a7){try{return `Object(`+ JSON.stringify(a)+ `)`}catch(a){return a7}};if(a instanceof $){return `${a.name}: ${a.message}\n${a.stack}`};return d});var g=(a=>{if(d===b.length)b.push(b.length+ Y);const c=d;d=b[c];b[c]=a;return c});var c=(a=>b[a]);var H=((a,b)=>{if(a===a0){return c(b)}else{return k(a,b)}});var F=((b,c,d)=>{a._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hcc69c5804cabb089(b,c,g(d))});var x=((b,c)=>{a._dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h84161d9efe89ee74(b,c)});var y=((b,c,d)=>{a._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h7064f03464f39ad4(b,c,g(d))});var o=((a,b,c)=>{if(c===W){const c=m.encode(a);const d=b(c.length,Y)>>>a0;j().subarray(d,d+ c.length).set(c);l=c.length;return d};let d=a.length;let e=b(d,Y)>>>a0;const f=j();let g=a0;for(;g<d;g++){const b=a.charCodeAt(g);if(b>127)break;f[e+ g]=b};if(g!==d){if(g!==a0){a=a.slice(g)};e=c(e,d,d=g+ a.length*3,Y)>>>a0;const b=j().subarray(e+ g,e+ d);const f=n(a,b);g+=f.written;e=c(e,d,g,Y)>>>a0};l=g;return e});var z=((b,c)=>{a._dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h3f4d0a624dc92b93(b,c)});var e=(a=>{if(a<132)return;b[a]=d;d=a});function G(b,c){try{return b.apply(this,c)}catch(b){a.__wbindgen_exn_store(g(b))}}var Q=(()=>{const b={};b.wbg={};b.wbg.__wbindgen_object_drop_ref=(a=>{f(a)});b.wbg.__wbindgen_object_clone_ref=(a=>{const b=c(a);return g(b)});b.wbg.__wbindgen_string_new=((a,b)=>{const c=k(a,b);return g(c)});b.wbg.__wbindgen_cb_drop=(a=>{const b=f(a).original;if(b.cnt--==Y){b.a=a0;return !0};const c=!1;return c});b.wbg.__wbindgen_is_undefined=(a=>{const b=c(a)===W;return b});b.wbg.__wbindgen_string_get=((b,d)=>{const e=c(d);const f=typeof e===a5?e:W;var g=p(f)?a0:o(f,a.__wbindgen_malloc,a.__wbindgen_realloc);var h=l;r()[b/aa+ Y]=h;r()[b/aa+ a0]=g});b.wbg.__wbindgen_is_string=(a=>{const b=typeof c(a)===a5;return b});b.wbg.__wbindgen_is_null=(a=>{const b=c(a)===X;return b});b.wbg.__wbindgen_jsval_eq=((a,b)=>{const d=c(a)===c(b);return d});b.wbg.__wbindgen_boolean_get=(a=>{const b=c(a);const d=typeof b===a4?(b?Y:a0):2;return d});b.wbg.__wbindgen_number_get=((a,b)=>{const d=c(b);const e=typeof d===a3?d:W;t()[a/8+ Y]=p(e)?a0:e;r()[a/aa+ a0]=!p(e)});b.wbg.__wbg_clearTimeout_541ac0980ffcef74=(a=>{const b=clearTimeout(f(a));return g(b)});b.wbg.__wbg_setTimeout_7d81d052875b0f4f=function(){return G(((a,b)=>{const d=setTimeout(c(a),b);return g(d)}),arguments)};b.wbg.__wbindgen_is_falsy=(a=>{const b=!c(a);return b});b.wbg.__wbindgen_number_new=(a=>{const b=a;return g(b)});b.wbg.__wbg_instanceof_Window_f401953a2cf86220=(a=>{let b;try{b=c(a) instanceof Window}catch(a){b=!1}const d=b;return d});b.wbg.__wbg_document_5100775d18896c16=(a=>{const b=c(a).document;return p(b)?a0:g(b)});b.wbg.__wbg_location_2951b5ee34f19221=(a=>{const b=c(a).location;return g(b)});b.wbg.__wbg_history_bc4057de66a2015f=function(){return G((a=>{const b=c(a).history;return g(b)}),arguments)};b.wbg.__wbg_origin_1caf0109f6508f7a=((b,d)=>{const e=c(d).origin;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/aa+ Y]=g;r()[b/aa+ a0]=f});b.wbg.__wbg_scrollTo_4d970c5e1c4b340b=((a,b,d)=>{c(a).scrollTo(b,d)});b.wbg.__wbg_requestAnimationFrame_549258cfa66011f0=function(){return G(((a,b)=>{const d=c(a).requestAnimationFrame(c(b));return d}),arguments)};b.wbg.__wbg_fetch_c4b6afebdb1f918e=((a,b)=>{const d=c(a).fetch(c(b));return g(d)});b.wbg.__wbg_body_edb1908d3ceff3a1=(a=>{const b=c(a).body;return p(b)?a0:g(b)});b.wbg.__wbg_createComment_354ccab4fdc521ee=((a,b,d)=>{var e=H(b,d);const f=c(a).createComment(e);return g(f)});b.wbg.__wbg_createDocumentFragment_8c86903bbb0a3c3c=(a=>{const b=c(a).createDocumentFragment();return g(b)});b.wbg.__wbg_createElement_8bae7856a4bb7411=function(){return G(((a,b,d)=>{var e=H(b,d);const f=c(a).createElement(e);return g(f)}),arguments)};b.wbg.__wbg_createTextNode_0c38fd80a5b2284d=((a,b,d)=>{var e=H(b,d);const f=c(a).createTextNode(e);return g(f)});b.wbg.__wbg_getElementById_c369ff43f0db99cf=((a,b,d)=>{var e=H(b,d);const f=c(a).getElementById(e);return p(f)?a0:g(f)});b.wbg.__wbg_classList_1f0528ee002e56d4=(a=>{const b=c(a).classList;return g(b)});b.wbg.__wbg_setinnerHTML_26d69b59e1af99c7=((a,b,d)=>{var e=H(b,d);c(a).innerHTML=e});b.wbg.__wbg_getAttribute_99bddb29274b29b9=((b,d,e,f)=>{var g=H(e,f);const h=c(d).getAttribute(g);var i=p(h)?a0:o(h,a.__wbindgen_malloc,a.__wbindgen_realloc);var j=l;r()[b/aa+ Y]=j;r()[b/aa+ a0]=i});b.wbg.__wbg_hasAttribute_8340e1a2a46f10f3=((a,b,d)=>{var e=H(b,d);const f=c(a).hasAttribute(e);return f});b.wbg.__wbg_removeAttribute_1b10a06ae98ebbd1=function(){return G(((a,b,d)=>{var e=H(b,d);c(a).removeAttribute(e)}),arguments)};b.wbg.__wbg_scrollIntoView_0c1a31f3d0dce6ae=(a=>{c(a).scrollIntoView()});b.wbg.__wbg_setAttribute_3c9f6c303b696daa=function(){return G(((a,b,d,e,f)=>{var g=H(b,d);var h=H(e,f);c(a).setAttribute(g,h)}),arguments)};b.wbg.__wbg_before_210596e44d88649f=function(){return G(((a,b)=>{c(a).before(c(b))}),arguments)};b.wbg.__wbg_remove_49b0a5925a04b955=(a=>{c(a).remove()});b.wbg.__wbg_append_fcf463f0b4a8f219=function(){return G(((a,b)=>{c(a).append(c(b))}),arguments)};b.wbg.__wbg_style_c3fc3dd146182a2d=(a=>{const b=c(a).style;return g(b)});b.wbg.__wbg_target_2fc177e386c8b7b0=(a=>{const b=c(a).target;return p(b)?a0:g(b)});b.wbg.__wbg_defaultPrevented_cc14a1dd3dd69c38=(a=>{const b=c(a).defaultPrevented;return b});b.wbg.__wbg_cancelBubble_c0aa3172524eb03c=(a=>{const b=c(a).cancelBubble;return b});b.wbg.__wbg_composedPath_58473fd5ae55f2cd=(a=>{const b=c(a).composedPath();return g(b)});b.wbg.__wbg_preventDefault_b1a4aafc79409429=(a=>{c(a).preventDefault()});b.wbg.__wbg_ctrlKey_008695ce60a588f5=(a=>{const b=c(a).ctrlKey;return b});b.wbg.__wbg_shiftKey_1e76dbfcdd36a4b4=(a=>{const b=c(a).shiftKey;return b});b.wbg.__wbg_altKey_07da841b54bd3ed6=(a=>{const b=c(a).altKey;return b});b.wbg.__wbg_metaKey_86bfd3b0d3a8083f=(a=>{const b=c(a).metaKey;return b});b.wbg.__wbg_button_367cdc7303e3cf9b=(a=>{const b=c(a).button;return b});b.wbg.__wbg_addEventListener_53b787075bd5e003=function(){return G(((a,b,d,e)=>{var f=H(b,d);c(a).addEventListener(f,c(e))}),arguments)};b.wbg.__wbg_addEventListener_4283b15b4f039eb5=function(){return G(((a,b,d,e,f)=>{var g=H(b,d);c(a).addEventListener(g,c(e),c(f))}),arguments)};b.wbg.__wbg_dispatchEvent_63c0c01600a98fd2=function(){return G(((a,b)=>{const d=c(a).dispatchEvent(c(b));return d}),arguments)};b.wbg.__wbg_removeEventListener_92cb9b3943463338=function(){return G(((a,b,d,e)=>{var f=H(b,d);c(a).removeEventListener(f,c(e))}),arguments)};b.wbg.__wbg_new_ab6fd82b10560829=function(){return G((()=>{const a=new Headers();return g(a)}),arguments)};b.wbg.__wbg_instanceof_HtmlAnchorElement_5fc0eb2fbc8672d8=(a=>{let b;try{b=c(a) instanceof HTMLAnchorElement}catch(a){b=!1}const d=b;return d});b.wbg.__wbg_target_f0876f510847bc60=((b,d)=>{const e=c(d).target;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/aa+ Y]=g;r()[b/aa+ a0]=f});b.wbg.__wbg_href_40fd5bca11c13133=((b,d)=>{const e=c(d).href;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/aa+ Y]=g;r()[b/aa+ a0]=f});b.wbg.__wbg_sethref_b94692d1a9f05b53=function(){return G(((a,b,d)=>{var e=H(b,d);c(a).href=e}),arguments)};b.wbg.__wbg_origin_ee93e29ace71f568=function(){return G(((b,d)=>{const e=c(d).origin;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/aa+ Y]=g;r()[b/aa+ a0]=f}),arguments)};b.wbg.__wbg_pathname_5449afe3829f96a1=function(){return G(((b,d)=>{const e=c(d).pathname;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/aa+ Y]=g;r()[b/aa+ a0]=f}),arguments)};b.wbg.__wbg_search_489f12953342ec1f=function(){return G(((b,d)=>{const e=c(d).search;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/aa+ Y]=g;r()[b/aa+ a0]=f}),arguments)};b.wbg.__wbg_hash_553098e838e06c1d=function(){return G(((b,d)=>{const e=c(d).hash;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/aa+ Y]=g;r()[b/aa+ a0]=f}),arguments)};b.wbg.__wbg_parentNode_6be3abff20e1a5fb=(a=>{const b=c(a).parentNode;return p(b)?a0:g(b)});b.wbg.__wbg_childNodes_118168e8b23bcb9b=(a=>{const b=c(a).childNodes;return g(b)});b.wbg.__wbg_previousSibling_9708a091a3e6e03b=(a=>{const b=c(a).previousSibling;return p(b)?a0:g(b)});b.wbg.__wbg_nextSibling_709614fdb0fb7a66=(a=>{const b=c(a).nextSibling;return p(b)?a0:g(b)});b.wbg.__wbg_settextContent_d271bab459cbb1ba=((a,b,d)=>{var e=H(b,d);c(a).textContent=e});b.wbg.__wbg_appendChild_580ccb11a660db68=function(){return G(((a,b)=>{const d=c(a).appendChild(c(b));return g(d)}),arguments)};b.wbg.__wbg_cloneNode_e19c313ea20d5d1d=function(){return G((a=>{const b=c(a).cloneNode();return g(b)}),arguments)};b.wbg.__wbg_length_d0a802565d17eec4=(a=>{const b=c(a).length;return b});b.wbg.__wbg_url_7807f6a1fddc3e23=((b,d)=>{const e=c(d).url;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/aa+ Y]=g;r()[b/aa+ a0]=f});b.wbg.__wbg_newwithstr_36b0b3f97efe096f=function(){return G(((a,b)=>{var c=H(a,b);const d=new Request(c);return g(d)}),arguments)};b.wbg.__wbg_newwithstrandinit_3fd6fba4083ff2d0=function(){return G(((a,b,d)=>{var e=H(a,b);const f=new Request(e,c(d));return g(f)}),arguments)};b.wbg.__wbg_instanceof_Response_849eb93e75734b6e=(a=>{let b;try{b=c(a) instanceof Response}catch(a){b=!1}const d=b;return d});b.wbg.__wbg_text_450a059667fd91fd=function(){return G((a=>{const b=c(a).text();return g(b)}),arguments)};b.wbg.__wbg_code_3b0c3912a2351163=((b,d)=>{const e=c(d).code;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/aa+ Y]=g;r()[b/aa+ a0]=f});b.wbg.__wbg_data_3ce7c145ca4fbcdc=(a=>{const b=c(a).data;return g(b)});b.wbg.__wbg_new_c7aa03c061e95bde=function(){return G((()=>{const a=new Range();return g(a)}),arguments)};b.wbg.__wbg_deleteContents_1b5a33e17bc6400f=function(){return G((a=>{c(a).deleteContents()}),arguments)};b.wbg.__wbg_setEndBefore_6d219390ff50f205=function(){return G(((a,b)=>{c(a).setEndBefore(c(b))}),arguments)};b.wbg.__wbg_setStartBefore_2dac025de1f18aa0=function(){return G(((a,b)=>{c(a).setStartBefore(c(b))}),arguments)};b.wbg.__wbg_byobRequest_72fca99f9c32c193=(a=>{const b=c(a).byobRequest;return p(b)?a0:g(b)});b.wbg.__wbg_close_184931724d961ccc=function(){return G((a=>{c(a).close()}),arguments)};b.wbg.__wbg_error_8e3928cfb8a43e2b=(a=>{console.error(c(a))});b.wbg.__wbg_log_5bb5f88f245d7762=(a=>{console.log(c(a))});b.wbg.__wbg_warn_63bbae1730aead09=(a=>{console.warn(c(a))});b.wbg.__wbg_state_9cc3f933b7d50acb=function(){return G((a=>{const b=c(a).state;return g(b)}),arguments)};b.wbg.__wbg_pushState_b8e8d346f8bb33fd=function(){return G(((a,b,d,e,f,g)=>{var h=H(d,e);var i=H(f,g);c(a).pushState(c(b),h,i)}),arguments)};b.wbg.__wbg_replaceState_ec9431bea5108a50=function(){return G(((a,b,d,e,f,g)=>{var h=H(d,e);var i=H(f,g);c(a).replaceState(c(b),h,i)}),arguments)};b.wbg.__wbg_close_a994f9425dab445c=function(){return G((a=>{c(a).close()}),arguments)};b.wbg.__wbg_enqueue_ea194723156c0cc2=function(){return G(((a,b)=>{c(a).enqueue(c(b))}),arguments)};b.wbg.__wbg_instanceof_WorkerGlobalScope_46b577f151fad960=(a=>{let b;try{b=c(a) instanceof WorkerGlobalScope}catch(a){b=!1}const d=b;return d});b.wbg.__wbg_fetch_921fad6ef9e883dd=((a,b)=>{const d=c(a).fetch(c(b));return g(d)});b.wbg.__wbg_removeProperty_fa6d48e2923dcfac=function(){return G(((b,d,e,f)=>{var g=H(e,f);const h=c(d).removeProperty(g);const i=o(h,a.__wbindgen_malloc,a.__wbindgen_realloc);const j=l;r()[b/aa+ Y]=j;r()[b/aa+ a0]=i}),arguments)};b.wbg.__wbg_setProperty_ea7d15a2b591aa97=function(){return G(((a,b,d,e,f)=>{var g=H(b,d);var h=H(e,f);c(a).setProperty(g,h)}),arguments)};b.wbg.__wbg_append_7ba9d5c2eb183eea=function(){return G(((a,b)=>{c(a).append(c(b))}),arguments)};b.wbg.__wbg_instanceof_ShadowRoot_9db040264422e84a=(a=>{let b;try{b=c(a) instanceof ShadowRoot}catch(a){b=!1}const d=b;return d});b.wbg.__wbg_host_c667c7623404d6bf=(a=>{const b=c(a).host;return g(b)});b.wbg.__wbg_href_7bfb3b2fdc0a6c3f=((b,d)=>{const e=c(d).href;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/aa+ Y]=g;r()[b/aa+ a0]=f});b.wbg.__wbg_origin_ea68ac578fa8517a=((b,d)=>{const e=c(d).origin;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/aa+ Y]=g;r()[b/aa+ a0]=f});b.wbg.__wbg_pathname_c5fe403ef9525ec6=((b,d)=>{const e=c(d).pathname;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/aa+ Y]=g;r()[b/aa+ a0]=f});b.wbg.__wbg_search_c68f506c44be6d1e=((b,d)=>{const e=c(d).search;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/aa+ Y]=g;r()[b/aa+ a0]=f});b.wbg.__wbg_setsearch_fd62f4de409a2bb3=((a,b,d)=>{var e=H(b,d);c(a).search=e});b.wbg.__wbg_searchParams_bc5845fe67587f77=(a=>{const b=c(a).searchParams;return g(b)});b.wbg.__wbg_hash_cdea7a9b7e684a42=((b,d)=>{const e=c(d).hash;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/aa+ Y]=g;r()[b/aa+ a0]=f});b.wbg.__wbg_new_67853c351755d2cf=function(){return G(((a,b)=>{var c=H(a,b);const d=new URL(c);return g(d)}),arguments)};b.wbg.__wbg_newwithbase_6aabbfb1b2e6a1cb=function(){return G(((a,b,c,d)=>{var e=H(a,b);var f=H(c,d);const h=new URL(e,f);return g(h)}),arguments)};b.wbg.__wbg_wasClean_8222e9acf5c5ad07=(a=>{const b=c(a).wasClean;return b});b.wbg.__wbg_code_5ee5dcc2842228cd=(a=>{const b=c(a).code;return b});b.wbg.__wbg_reason_5ed6709323849cb1=((b,d)=>{const e=c(d).reason;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/aa+ Y]=g;r()[b/aa+ a0]=f});b.wbg.__wbg_newwitheventinitdict_c939a6b964db4d91=function(){return G(((a,b,d)=>{var e=H(a,b);const f=new CloseEvent(e,c(d));return g(f)}),arguments)};b.wbg.__wbg_add_dcb05a8ba423bdac=function(){return G(((a,b,d)=>{var e=H(b,d);c(a).add(e)}),arguments)};b.wbg.__wbg_remove_698118fb25ab8150=function(){return G(((a,b,d)=>{var e=H(b,d);c(a).remove(e)}),arguments)};b.wbg.__wbg_new_4c501d7c115d20a6=function(){return G((()=>{const a=new URLSearchParams();return g(a)}),arguments)};b.wbg.__wbg_setbinaryType_b0cf5103cd561959=((a,b)=>{c(a).binaryType=f(b)});b.wbg.__wbg_new_6c74223c77cfabad=function(){return G(((a,b)=>{var c=H(a,b);const d=new WebSocket(c);return g(d)}),arguments)};b.wbg.__wbg_close_acd9532ff5c093ea=function(){return G((a=>{c(a).close()}),arguments)};b.wbg.__wbg_setdata_8c2b43af041cc1b3=((a,b,d)=>{var e=H(b,d);c(a).data=e});b.wbg.__wbg_view_7f0ce470793a340f=(a=>{const b=c(a).view;return p(b)?a0:g(b)});b.wbg.__wbg_respond_b1a43b2e3a06d525=function(){return G(((a,b)=>{c(a).respond(b>>>a0)}),arguments)};b.wbg.__wbg_queueMicrotask_3cbae2ec6b6cd3d6=(a=>{const b=c(a).queueMicrotask;return g(b)});b.wbg.__wbindgen_is_function=(a=>{const b=typeof c(a)===a2;return b});b.wbg.__wbg_queueMicrotask_481971b0d87f3dd4=(a=>{queueMicrotask(c(a))});b.wbg.__wbg_get_bd8e338fbd5f5cc8=((a,b)=>{const d=c(a)[b>>>a0];return g(d)});b.wbg.__wbg_length_cd7af8117672b8b8=(a=>{const b=c(a).length;return b});b.wbg.__wbg_newnoargs_e258087cd0daa0ea=((a,b)=>{var c=H(a,b);const d=new Function(c);return g(d)});b.wbg.__wbindgen_is_object=(a=>{const b=c(a);const d=typeof b===`object`&&b!==X;return d});b.wbg.__wbg_next_40fc327bfc8770e6=(a=>{const b=c(a).next;return g(b)});b.wbg.__wbg_next_196c84450b364254=function(){return G((a=>{const b=c(a).next();return g(b)}),arguments)};b.wbg.__wbg_done_298b57d23c0fc80c=(a=>{const b=c(a).done;return b});b.wbg.__wbg_value_d93c65011f51a456=(a=>{const b=c(a).value;return g(b)});b.wbg.__wbg_iterator_2cee6dadfd956dfa=(()=>{const a=Symbol.iterator;return g(a)});b.wbg.__wbg_get_e3c254076557e348=function(){return G(((a,b)=>{const d=ab.get(c(a),c(b));return g(d)}),arguments)};b.wbg.__wbg_call_27c0f87801dedf93=function(){return G(((a,b)=>{const d=c(a).call(c(b));return g(d)}),arguments)};b.wbg.__wbg_new_72fb9a18b5ae2624=(()=>{const a=new ac();return g(a)});b.wbg.__wbg_self_ce0dbfc45cf2f5be=function(){return G((()=>{const a=self.self;return g(a)}),arguments)};b.wbg.__wbg_window_c6fb939a7f436783=function(){return G((()=>{const a=window.window;return g(a)}),arguments)};b.wbg.__wbg_globalThis_d1e6af4856ba331b=function(){return G((()=>{const a=globalThis.globalThis;return g(a)}),arguments)};b.wbg.__wbg_global_207b558942527489=function(){return G((()=>{const a=global.global;return g(a)}),arguments)};b.wbg.__wbg_decodeURI_34e1afc7326c927c=function(){return G(((a,b)=>{var c=H(a,b);const d=decodeURI(c);return g(d)}),arguments)};b.wbg.__wbg_isArray_2ab64d95e09ea0ae=(a=>{const b=a6(c(a));return b});b.wbg.__wbg_instanceof_ArrayBuffer_836825be07d4c9d2=(a=>{let b;try{b=c(a) instanceof ArrayBuffer}catch(a){b=!1}const d=b;return d});b.wbg.__wbg_instanceof_Error_e20bb56fd5591a93=(a=>{let b;try{b=c(a) instanceof $}catch(a){b=!1}const d=b;return d});b.wbg.__wbg_new_28c511d9baebfa89=((a,b)=>{var c=H(a,b);const d=new $(c);return g(d)});b.wbg.__wbg_message_5bf28016c2b49cfb=(a=>{const b=c(a).message;return g(b)});b.wbg.__wbg_name_e7429f0dda6079e2=(a=>{const b=c(a).name;return g(b)});b.wbg.__wbg_toString_ffe4c9ea3b3532e9=(a=>{const b=c(a).toString();return g(b)});b.wbg.__wbg_call_b3ca7c6051f9bec1=function(){return G(((a,b,d)=>{const e=c(a).call(c(b),c(d));return g(e)}),arguments)};b.wbg.__wbg_is_010fdc0f4ab96916=((a,b)=>{const d=ac.is(c(a),c(b));return d});b.wbg.__wbg_toString_c816a20ab859d0c1=(a=>{const b=c(a).toString();return g(b)});b.wbg.__wbg_exec_b9996525463e30df=((a,b,d)=>{var e=H(b,d);const f=c(a).exec(e);return p(f)?a0:g(f)});b.wbg.__wbg_new_3c970fa9da0c5794=((a,b,c,d)=>{var e=H(a,b);var f=H(c,d);const h=new RegExp(e,f);return g(h)});b.wbg.__wbg_new_81740750da40724f=((a,b)=>{try{var c={a:a,b:b};var d=(a,b)=>{const d=c.a;c.a=a0;try{return I(d,c.b,a,b)}finally{c.a=d}};const e=new ad(d);return g(e)}finally{c.a=c.b=a0}});b.wbg.__wbg_resolve_b0083a7967828ec8=(a=>{const b=ad.resolve(c(a));return g(b)});b.wbg.__wbg_then_0c86a60e8fcfe9f6=((a,b)=>{const d=c(a).then(c(b));return g(d)});b.wbg.__wbg_then_a73caa9a87991566=((a,b,d)=>{const e=c(a).then(c(b),c(d));return g(e)});b.wbg.__wbg_buffer_12d079cc21e14bdb=(a=>{const b=c(a).buffer;return g(b)});b.wbg.__wbg_newwithbyteoffsetandlength_aa4a17c33a06e5cb=((a,b,d)=>{const e=new a1(c(a),b>>>a0,d>>>a0);return g(e)});b.wbg.__wbg_new_63b92bc8671ed464=(a=>{const b=new a1(c(a));return g(b)});b.wbg.__wbg_set_a47bac70306a19a7=((a,b,d)=>{c(a).set(c(b),d>>>a0)});b.wbg.__wbg_length_c20a40f15020d68a=(a=>{const b=c(a).length;return b});b.wbg.__wbg_buffer_dd7f74bc60f1faab=(a=>{const b=c(a).buffer;return g(b)});b.wbg.__wbg_byteLength_58f7b4fab1919d44=(a=>{const b=c(a).byteLength;return b});b.wbg.__wbg_byteOffset_81d60f7392524f62=(a=>{const b=c(a).byteOffset;return b});b.wbg.__wbg_set_1f9b04f170055d33=function(){return G(((a,b,d)=>{const e=ab.set(c(a),c(b),c(d));return e}),arguments)};b.wbg.__wbindgen_debug_string=((b,d)=>{const e=u(c(d));const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/aa+ Y]=g;r()[b/aa+ a0]=f});b.wbg.__wbindgen_throw=((a,b)=>{throw new $(k(a,b))});b.wbg.__wbindgen_memory=(()=>{const b=a.memory;return g(b)});b.wbg.__wbindgen_closure_wrapper897=((a,b,c)=>{const d=w(a,b,ae,x);return g(d)});b.wbg.__wbindgen_closure_wrapper899=((a,b,c)=>{const d=w(a,b,ae,y);return g(d)});b.wbg.__wbindgen_closure_wrapper1432=((a,b,c)=>{const d=w(a,b,514,z);return g(d)});b.wbg.__wbindgen_closure_wrapper1477=((a,b,c)=>{const d=w(a,b,538,A);return g(d)});b.wbg.__wbindgen_closure_wrapper1626=((a,b,c)=>{const d=w(a,b,581,B);return g(d)});b.wbg.__wbindgen_closure_wrapper1670=((a,b,c)=>{const d=w(a,b,af,C);return g(d)});b.wbg.__wbindgen_closure_wrapper1672=((a,b,c)=>{const d=w(a,b,af,C);return g(d)});b.wbg.__wbindgen_closure_wrapper1674=((a,b,c)=>{const d=w(a,b,af,C);return g(d)});b.wbg.__wbindgen_closure_wrapper1676=((a,b,c)=>{const d=w(a,b,af,D);return g(d)});b.wbg.__wbindgen_closure_wrapper1815=((a,b,c)=>{const d=w(a,b,653,E);return g(d)});b.wbg.__wbindgen_closure_wrapper3205=((a,b,c)=>{const d=w(a,b,685,F);return g(d)});return b});var j=(()=>{if(i===X||i.byteLength===a0){i=new a1(a.memory.buffer)};return i});var B=((b,c,d)=>{a._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h6c55c62e7cb0b915(b,c,g(d))});var w=((b,c,d,e)=>{const f={a:b,b:c,cnt:Y,dtor:d};const g=(...b)=>{f.cnt++;const c=f.a;f.a=a0;try{return e(c,f.b,...b)}finally{if(--f.cnt===a0){a.__wbindgen_export_2.get(f.dtor)(c,f.b);v.unregister(f)}else{f.a=c}}};g.original=f;v.register(g,f,f);return g});var P=(async(a,b)=>{if(typeof Response===a2&&a instanceof Response){if(typeof WebAssembly.instantiateStreaming===a2){try{return await WebAssembly.instantiateStreaming(a,b)}catch(b){if(a.headers.get(`Content-Type`)!=`application/wasm`){console.warn(`\`WebAssembly.instantiateStreaming\` failed because your server does not serve wasm with \`application/wasm\` MIME type. Falling back to \`WebAssembly.instantiate\` which is slower. Original error:\\n`,b)}else{throw b}}};const c=await a.arrayBuffer();return await WebAssembly.instantiate(c,b)}else{const c=await WebAssembly.instantiate(a,b);if(c instanceof WebAssembly.Instance){return {instance:c,module:a}}else{return c}}});var t=(()=>{if(s===X||s.byteLength===a0){s=new Float64Array(a.memory.buffer)};return s});var p=(a=>a===W||a===X);var r=(()=>{if(q===X||q.byteLength===a0){q=new Int32Array(a.memory.buffer)};return q});var k=((a,b)=>{a=a>>>a0;return h.decode(j().subarray(a,a+ b))});var S=((b,c)=>{a=b.exports;U.__wbindgen_wasm_module=c;s=X;q=X;i=X;a.__wbindgen_start();return a});var I=((b,c,d,e)=>{a.wasm_bindgen__convert__closures__invoke2_mut__h8c6e8744b4c60252(b,c,g(d),g(e))});var D=((b,c)=>{a._dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h2d6f7a6842802731(b,c)});var U=(async(b)=>{if(a!==W)return a;if(typeof b===Z){b=new URL(`client-v2_bg.wasm`,import.meta.url)};const c=Q();if(typeof b===a5||typeof Request===a2&&b instanceof Request||typeof URL===a2&&b instanceof URL){b=fetch(b)};R(c);const {instance:d,module:e}=await P(await b,c);return S(d,e)});var T=(b=>{if(a!==W)return a;const c=Q();R(c);if(!(b instanceof WebAssembly.Module)){b=new WebAssembly.Module(b)};const d=new WebAssembly.Instance(b,c);return S(d,b)});let a;const b=new V(128).fill(W);b.push(W,X,!0,!1);let d=b.length;const h=typeof TextDecoder!==Z?new TextDecoder(_,{ignoreBOM:!0,fatal:!0}):{decode:()=>{throw $(`TextDecoder not available`)}};if(typeof TextDecoder!==Z){h.decode()};let i=X;let l=a0;const m=typeof TextEncoder!==Z?new TextEncoder(_):{encode:()=>{throw $(`TextEncoder not available`)}};const n=typeof m.encodeInto===a2?((a,b)=>m.encodeInto(a,b)):((a,b)=>{const c=m.encode(a);b.set(c);return {read:a.length,written:c.length}});let q=X;let s=X;const v=typeof a8===Z?{register:()=>{},unregister:()=>{}}:new a8(b=>{a.__wbindgen_export_2.get(b.dtor)(b.a,b.b)});const J=typeof a8===Z?{register:()=>{},unregister:()=>{}}:new a8(b=>a.__wbg_intounderlyingbytesource_free(b>>>a0));class K{__destroy_into_raw(){const a=this.__wbg_ptr;this.__wbg_ptr=a0;J.unregister(this);return a}free(){const b=this.__destroy_into_raw();a.__wbg_intounderlyingbytesource_free(b)}type(){try{const e=a.__wbindgen_add_to_stack_pointer(-a9);a.intounderlyingbytesource_type(e,this.__wbg_ptr);var b=r()[e/aa+ a0];var c=r()[e/aa+ Y];var d=H(b,c);if(b!==a0){a.__wbindgen_free(b,c,Y)};return d}finally{a.__wbindgen_add_to_stack_pointer(a9)}}autoAllocateChunkSize(){const b=a.intounderlyingbytesource_autoAllocateChunkSize(this.__wbg_ptr);return b>>>a0}start(b){a.intounderlyingbytesource_start(this.__wbg_ptr,g(b))}pull(b){const c=a.intounderlyingbytesource_pull(this.__wbg_ptr,g(b));return f(c)}cancel(){const b=this.__destroy_into_raw();a.intounderlyingbytesource_cancel(b)}}const L=typeof a8===Z?{register:()=>{},unregister:()=>{}}:new a8(b=>a.__wbg_intounderlyingsink_free(b>>>a0));class M{__destroy_into_raw(){const a=this.__wbg_ptr;this.__wbg_ptr=a0;L.unregister(this);return a}free(){const b=this.__destroy_into_raw();a.__wbg_intounderlyingsink_free(b)}write(b){const c=a.intounderlyingsink_write(this.__wbg_ptr,g(b));return f(c)}close(){const b=this.__destroy_into_raw();const c=a.intounderlyingsink_close(b);return f(c)}abort(b){const c=this.__destroy_into_raw();const d=a.intounderlyingsink_abort(c,g(b));return f(d)}}const N=typeof a8===Z?{register:()=>{},unregister:()=>{}}:new a8(b=>a.__wbg_intounderlyingsource_free(b>>>a0));class O{__destroy_into_raw(){const a=this.__wbg_ptr;this.__wbg_ptr=a0;N.unregister(this);return a}free(){const b=this.__destroy_into_raw();a.__wbg_intounderlyingsource_free(b)}pull(b){const c=a.intounderlyingsource_pull(this.__wbg_ptr,g(b));return f(c)}cancel(){const b=this.__destroy_into_raw();a.intounderlyingsource_cancel(b)}}export default U;export{K as IntoUnderlyingByteSource,M as IntoUnderlyingSink,O as IntoUnderlyingSource,T as initSync}