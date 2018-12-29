
{
  var a = require("./");
  console.log(a);

  console.log("TEST1:");
  try {
    console.log(a.test1(123, "sdsdf"))
    console.log(a.test1(123))
  } catch(e) {
    console.log("JS:", e);
  }
  //console.log(a.test1("sebastien chapuis"))
  console.log("SUPER:");
  console.log(a.my_super_function("seb", 1234))
  console.log("\nOTHER:");
  console.log(a.my_other_function("seb", 1234, "coucou toi"))
  //console.log("\TEST4:");
  console.log(a.test4("seb", 1234, "coucou toi"))
  //console.log("\TEST4:");
  console.log(a.test5("seb", 1234, "coucou toi"))
  console.log(a.test6("seb", 1234, "coucou toi"))
  console.log(a.test7(93))
  console.log(a.test8(93))
  //let myclass4 = a.MySuperClass("salut toi", 25);
  console.log(a.test8())
  let myclass = new a.someclass("salut", 123423423493);
  //let myclass = a.someclass("salut", 123423423493);
  console.log("class:", a.someclass)
  console.log("class:", myclass)
  //console.log("class:", myclass.a_function())
  //  console.log("class:", myclass.easy())
  //myclass.easy3 = undefined;
  console.log("easy3:", myclass.easy3)
  console.log("data:", myclass.__pinar_class_data);
  myclass.__pinar_class_data = undefined;
  delete myclass.__pinar_class_data;
  console.log("data:", myclass.__pinar_class_data);
  //console.log("EASY:", myclass.easy(234));
  console.log("EASY:", myclass.easy("seb", 234));
  // console.log("EASY2:", myclass.easy2("sdfs", 234));
  // console.log("EASY3:", myclass.easy3);
  let myclass2 = new a.someclass("salut seb", 10);
  console.log("EASY:", myclass2.easy("seb", 234));
  console.log("EASY:", myclass.easy("seb", 234));
  console.log("EASY:", myclass.easy4);
  //console.log("STRUCT:", a.test9());
  console.log("BOX:", a.test10());
  console.log("11:", a.test11("salut", {}));
  console.log("12:", a.test12("salut", {}));
  console.log("13:", a.test13({ a: 1, b: 2, c: 3, d: { A: "wesh" }}));
  console.log("13:", a.test13({ a: 1, b: 2, c: 3, d: { B: 91 }}));
  console.log("13:", a.test13({ a: 1, b: 2, c: 3, d: { C: [5, 6, 7] }}));
  console.log("13:", a.test13({ a: 1, b: 2, c: 3, d: { D: 2 }}));
  console.log("13:", a.test13({ a: 1, b: 2, c: 3, d: { E: 10 }}));
  try {
    console.log("13:", a.test13({ a: "seb", b: 2, c: 3, d: { E: 10 }}));
  } catch (e) { console.log(e); }
  try {
    console.log("13:", a.test13({ a: 1, b: 2, c: 3 }));
  } catch (e) { console.log(e); }
  try {
    console.log("13:", a.test13({ a: 1, b: 2, c: 3, d: { E: 10 }, e: { s: "seb" } }));
  } catch (e) { console.log(e); }
  //let myclass3 = a.someclass("salut toi", 25);
  //console.log("EASY:", myclass3);
  console.log("14:", a.test14());
  const res_15 = a.test15({ a: 55 });
  console.log("15:", res_15, res_15.wesh("salut"));
  //console.log(a.test16((a) => console.log(`ARG:`, a)));
}

// for (let i = 0; i < 100; i+= 1) {
//   let myclass2 = new a.someclass("salut seb", 10);
//   //myclass2.easy("seb", 1234);
//   console.log("EASY:", myclass2.easy("seb", 234), i);
// }
// console.log("LAAAA");
// global.gc();

//setTimeout(() => console.log("done"), 1000);
//global.gc();
//setTimeout(() => console.log("done"), 1000);
