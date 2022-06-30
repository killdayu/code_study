# 一个引例

```rust
use std::io; //导入库，use

fn main() {
    println!("Guess the number!");
    println!("Please input your guess.");

    let mut guess = String::new();	//可变变量，mut
    io::stdin().read_line(&mut guess)	//pub fn read_line(&self, buf: &mut String) -> io::Result<usize>
        .expect("Failed to read line");

    println!("You guessed: {}", guess);
}
```

String::new中的::语法表明new是String类型的一个关联函数（associated function）。我们会针对类型本身来定义关联函数，比如本例中的String，而不会针对String的某个特定实例。关联函数在某些语言中也被称为静态方法（static method）。

```rust
use std::io;
use rand::Rng;	//这里的Rng是一个trait（特征），它定义了随机数生成器需要实现的方法集合。

fn main() {
    println!("Guess the number!");

    let secret_number = rand::thread_rng().gen_range(1,101);

    println!("The secret number is : {}", secret_number);

    println!("Please input your guess.");

    let mut guess = String::new();
    io::stdin().read_line(&mut guess)
        .expect("Failed to read line");

    println!("You guessed: {}", guess);
}

```

```rust
use std::cmp::Ordering;
use std::io;
use rand::Rng;

fn main() {
    println!("Guess the number!");

    let secret_number = rand::thread_rng().gen_range(1,101);

    println!("The secret number is : {}", secret_number);

    println!("Please input your guess.");

    let mut guess = String::new();
    io::stdin().read_line(&mut guess)
        .expect("Failed to read line");

    println!("You guessed: {}", guess);

    match guess.cmp(&secret_number) {	//fn cmp(&self, other: &Self) -> Ordering
        Ordering::Less => println!("Too small!"),	//arm，pattern
        Ordering::Greater => println!("Too big!"),
        Ordering::Equal => println!("You win!"),
    }
}
```

这里出现了一行新的use声明 ，它从标准库中引入了一个名为std::cmp::Ordering的类型。与Result相同，Ordering也是一个枚举类型，它拥有Less、Greater及Equal这3个变体。它们分别被用来表示比较两个数字之后可能产生的3种结果。

match表达式由数个分支 （arm）组成，每个分支都包含一个用于匹配的模式 （pattern），以及匹配成功后要执行的相应的代码。Rust会尝试用我们传入match表达式的值去依次匹配每个分支的模式，一旦匹配成功，它就会执行当前分支中的代码。

```rust
use std::cmp::Ordering;
use std::io;
use rand::Rng;

fn main() {
    println!("Guess the number!");

    let secret_number = rand::thread_rng().gen_range(1,101);

    println!("The secret number is : {}", secret_number);

    println!("Please input your guess.");

    let mut guess = String::new();
    io::stdin().read_line(&mut guess)
        .expect("Failed to read line");

    let guess: u32 = guess.trim().parse()	//隐藏机制，trim去除首尾空白字符，parse将字符串解析为数值（let guess: u32）
        .expect("Please type a number!");

    println!("You guessed: {}", guess);

    match guess.cmp(&secret_number) {
        Ordering::Less => println!("Too small!"),
        Ordering::Greater => println!("Too big!"),
        Ordering::Equal => println!("You win!"),
    }
}
```

```rust
use std::cmp::Ordering;
use std::io;
use rand::Rng;

fn main() {
    println!("Guess the number!");

    let secret_number = rand::thread_rng().gen_range(1, 101);

    println!("The secret number is : {}", secret_number);

    loop {
        println!("Please input your guess.");

        let mut guess = String::new();
        io::stdin().read_line(&mut guess)
            .expect("Failed to read line");

        let guess: u32 = guess.trim().parse()
            .expect("Please type a number!");

        println!("You guessed: {}", guess);

        match guess.cmp(&secret_number) {
            Ordering::Less => println!("Too small!"),
            Ordering::Greater => println!("Too big!"),
            Ordering::Equal => {
                println!("You win!");
                break;	//退出循环，break
            },
        }
    }
}
```

```rust
use std::cmp::Ordering;
use std::io;
use rand::Rng;

fn main() {
    println!("Guess the number!");

    let secret_number = rand::thread_rng().gen_range(1, 101);

    println!("The secret number is : {}", secret_number);

    loop {
        println!("Please input your guess.");

        let mut guess = String::new();
        io::stdin().read_line(&mut guess)
            .expect("Failed to read line");

        let guess: u32 = match guess.trim().parse() {	//pub fn parse<F: FromStr>(&self) -> Result<F, F::Err>
            Ok(num) => num,
            Err(_) => continue,
        };

        println!("You guessed: {}", guess);

        match guess.cmp(&secret_number) {
            Ordering::Less => println!("Too small!"),
            Ordering::Greater => println!("Too big!"),
            Ordering::Equal => {
                println!("You win!");
                break;
            }
        }
    }
}
```

# 一些基本概念

## 基本数据类型

```rust
const Z: u32 = 0;   //必须显示声明常量的类型

fn main() {
    let x = 0;  //变量默认不可变
    println!("The value of x is : {}", x);

    let x = 1.1;  //隐藏机制，复用变量名同时改变类型。
    println!("The value of x is : {}", x);


    let mut y = 0;  //声明可变变量，mut
    println!("The value of y is : {}", y);

    y = 1;
    println!("The value of y is : {}", y);


    let test: u8 = 255; //新版本golang不会触发环绕，--release
    println!("The value of test is : {}", test);

    let test1 = Z * y;  //类型相同才能运算
    println!("The value of test1 is : {}", test1);

    let test2 = true;   //bool
    println!("The value of test2 is : {}", test2);

    let test3 = '🐱';    //char，unicode编码
    println!("The value of test3 is : {}", test3);

    //复合类型

    let tup: (i32, f64, u8) = (500, 6.4, 1);    //元组，tuple，存储不同类型的数据

    let(_x, _y, _z) =tup;   //解构

    println!("The value of tup_x is : {}", _x);
    println!("The value of tup_x is : {}", tup.0);  //通过索引访问

    let a = [1, 2, 3, 4, 5];
    let a1 = [3; 5];    //等价于let a1 = [3, 3, 3, 3, 3];

    println!("The value of a is : {}", a[0]);
    println!("The value of a1 is : {}", a1[0]);
    println!("The value of a is : {}", a[4]);   //数组越界访问，报错，无法编译。
    
}
```

## 函数，语句，表达式

```rust
fn main() {
    println!("Hello, world!");

    another_function(); //调用自定义函数
    another_function_1(0, true);  //传递实参

    let x = {
        let y = 0;
        y + 1
    };  //{}也是个表达式，返回值为2，表达式不能加;，末尾这个;是let的。
    println!("The value of x is : {}" , x);   //x = 2，这里没有y（不在作用域）。

    let z = another_function_2();
    println!("The value of z is : {}" , z);
}

fn another_function() {
    println!("Another function.")
}

fn another_function_1(x: i32, y: bool) { //必须显式声明形参
    println!("The value of x,y is : {}, {}", x, y);
}

fn another_function_2() -> char {
    '🐱'
}
```

## 条件判断

```rust
fn main() {
    let number = 5;

    if number < 5 {
        println!("condition was true");
    } else if number ==5 {
        println!("yes");
    } else {
        println!("condition was false");
    }

    let condition = true;
    let number = if condition {
        5
    } else {
        6
    };
    println!("The value of number is: {}", number);

}
```

## 循环

```rust
fn main() {
    let mut counter = 0;

    let result = loop {
        counter += 1;
        //println!("{}", counter);

        if counter == 10 {
            break counter * 2;
        }
    };
    println!("The result is {}", result);

    let mut number = 3;

    while number != 0 {
        println!("{}!", number);

        number = number - 1;
    }
    println!("LIFTOFF!!!");

    let a = [10, 20, 30, 40, 50];
    let mut index = 0;
    while index < 5 {
        println!("the value is : {}", a[index]);    //此时index隐式声明的类型为usize
        index = index + 1;
    }

    for element in a.iter() {
        println!("the value is : {}", element);
    }

    for number in (1..4).rev() {    //(1..4)语法糖？fn rev(self) -> Rev<Self>
        println!("{}!", number);
    }
    println!("LIFTOFF!!!");
}
```

#  所有权

所有权可以说是Rust中最为独特的一个功能了。正是所有权概念和相关工具的引入，Rust才能够在没有垃圾回收机制的前提下保障内存安全。

## 栈与堆

在许多编程语言中，程序员不需要频繁地考虑栈空间和堆空间的区别。但对于Rust这样的系统级编程语言来说，一个值被存储在栈还是被存储在堆上会极大地影响到语言的行为，进而影响到我们编写代码时的设计抉择。由于所有权的某些内容会涉及栈与堆，所以让我们先来简单地了解一下它们。

栈和堆都是代码在运行时可以使用的内存空间，不过它们通常以不同的结构组织而成。栈会以我们放入值时的顺序来存储它们，并以相反的顺序将值取出。这也就是所谓的“后进先出”策略。你可以把栈上的操作想象成堆放盘子：当你需要放置盘子时，你只能将它们放置在栈的顶部，而当你需要取出盘子时，你也只能从顶部取出。换句话说，你没有办法从中间或底部插入或移除盘子。用术语来讲，添加数据这一操作被称作入栈，移除数据则被称作出栈。所有存储在栈中的数据都必须拥有一个已知且固定的大小。对于那些在编译期无法确定大小的数据，你就只能将它们存储在堆中。堆空间的管理是较为松散的：当你希望将数据放入堆中时，你就可以请求特定大小的空间。操作系统会根据你的请求在堆中找到一块足够大的可用空间，将它标记为已使用，并把指向这片空间地址的指针返回给我们。这一过程就是所谓的堆分配，它也常常被简称为分配。将值压入栈中不叫分配。由于指针的大小是固定的且可以在编译期确定，所以可以将指针存储在栈中。当你想要访问指针所指向的具体数据时，可以通过指针指向的地址来访问。你可以把这个过程想象为到餐厅聚餐。当你到达餐厅表明自己需要的座位数后，服务员会找到一张足够大的空桌子，并将你们领过去入座。即便这时有小伙伴来迟了，他们也可以通过询问你们就座的位置来找到你们。向栈上推入数据要比在堆上进行分配更有效率一些，因为操作系统省去了搜索新数据存储位置的工作；这个位置永远处于栈的顶端。除此之外，操作系统在堆上分配空间时还必须首先找到足够放下对应数据的空间，并进行某些记录工作来协调随后进行的其余分配操作。由于多了指针跳转的环节，所以访问堆上的数据要慢于访问栈上的数据。一般来说，现代处理器在进行计算的过程中，由于缓存的缘故，指令在内存中跳转的次数越多，性能就越差。继续使用上面的餐厅来作类比。假设现在同时有许多桌的顾客正在等待服务员的处理。那么最高效的处理方式自然是报完一张桌子所有的订单后再接着服务下一张桌子的顾客。而一旦服务员每次在单个桌子前只处理单个订单，那么他就不得不浪费较多的时间往返于不同的桌子之间。出于同样的原因，处理器在操作排布紧密的数据（比如在栈上）时要比操作排布稀疏的数据（比如在堆上）有效率得多。另外，分配命令本身也可能消耗不少时钟周期。许多系统编程语言都需要你记录代码中分配的堆空间，最小化堆上的冗余数据，并及时清理堆上的无用数据以避免耗尽空间。而所有权概念则解决了这些问题。一旦你熟练地掌握了所有权及其相关工具，就可以将这些问题交给Rust处理，减轻用于思考栈和堆的心智负担。不过，知晓如何使用和管理堆内存可以帮助我们理解所有权存在的意义及其背后的工作原理。

**将堆的指针存放在栈内。**

## 所有权规则

- Rust中的每一个值都有一个对应的变量作为它的所有者 。
- 在同一时间内，值有且仅有一个所有者。
- 当所有者离开自己的作用域时，它持有的值就会被释放掉。

![](https://raw.githubusercontent.com/killdayu/killdayu.github.io_file/main/pictures/Image%20From%20rust%E6%9D%83%E5%A8%81%E6%8C%87%E5%8D%97.png)

--

![](https://raw.githubusercontent.com/killdayu/killdayu.github.io_file/main/pictures/Image%20From%20rust%E6%9D%83%E5%A8%81%E6%8C%87%E5%8D%97_1.png)

```rust
fn main() { //由于变量s还末被声明，所以它在这里是不可用的
    let s = "hello";    //从这里开始变量s变得可用
    let mut ss = String::from("hello");

    ss.push_str(", world!");
    //println!("{}", ss);

    takes_ownership(ss);    //进入作用域，浅拷贝
    //println!("after, {}", ss);    //ss已经被释放了
    let x= 0;   //Copy
    makes_copy(x);
    println!("after, {}", x);

}   //作用城到这里结束，变量s再次不可用

fn takes_ownership(some_string: String) {
    println!("fn, {}", some_string);
}   //drop函数调用，回收堆，ss不存在。

fn makes_copy(some_integer: i32) {
    println!("fn, {}", some_integer);
}
```

## 引用

```rust
fn main() {
    let s = String::from("hello");
    let s_len = string_len(&s);
    println!("s is: {}, s_len is: {}", s, s_len);  //s的所有权还在。
    println!("s addr is: {:p}", &s);

    let ps = &s;
    println!("ps is: {:p}", ps);
    println!("ps addr is: {:p}", &ps);
}

fn string_len(s:&String) -> usize{
    s.len()
}
```

&String替代了String，并没有产生浅拷贝，复制的是String的地址而已，所以String的所有权不会转移，所以就不可能被Drop。

## 可变引用

```rust
fn main() {
    let mut s = String::from("hello");
 
     let r1 = &s;
     let r2 = &s;
     println!("{} and {}", r1, r2); // 新编译器中，r1,r2作用域在这里结束
 
     let r3 = &mut s;
     println!("{}", r3);    // 新编译器中，r3作用域在这里结束
     let r4 = &mut s;
     println!("{}", r4);    // 新编译器中，r4作用域在这里结束
 }  // 老编译器中，r1、r2、r3作用域在这里结束
```

在老版本的编译器中（Rust 1.31 前），将会报错，因为 `r1 和 `r2` 的作用域在花括号 `}` 处结束，那么 `r3` 的借用就会触发 **无法同时借用可变和不可变**的规则。

但是在新的编译器中，该代码将顺利通过，因为 **引用作用域的结束位置从花括号变成最后一次使用的位置**，因此 `r1` 借用和 `r2` 借用在 `println!` 后，就结束了，此时 `r3` 可以顺利借用到可变引用。

**注意，引用`(r1,r2,r3)`的作用域从创建开始，一直持续到它最后一次使用的地方`(println!(....)行)`，这个跟变量的作用域有所不同，变量的作用域从创建持续到某一个花括号 }**

### NLL

对于这种编译器优化行为，Rust 专门起了一个名字 —— **Non-Lexical Lifetimes(NLL)**，专门用于找到某个引用在作用域(`}`)结束前就不再被使用的代码位置。

虽然这种借用错误有的时候会让我们很郁闷，但是你只要想想这是 Rust 提前帮你发现了潜在的 BUG，其实就开心了，虽然减慢了开发速度，但是从长期来看，大幅减少了后续开发和运维成本。

### debug

在查找资料的过程中，我发现这么一段代码，如下：

```rust
fn main() {
    let mut s = String::from("hello");
    let s1 = &mut s;
    println!("{}", s);
    println!("{}", s1);
}
```

编译失败，但是这样则会编译成功，去掉println!也能编译成功。

```rust
fn main() {
    let mut s = String::from("hello");
    let s1 = &mut s;
    println!("{}", s1);
    println!("{}", s);
}
```

从代码来看，这仅仅只是2个打印语句的位置交换，其实涉及到了可变引用的基本规则。

• 在任何一段给定的时间里，你要么只能拥有一个可变引用，要么只能拥有任意数量的不可变引用。
• 引用总是有效的。

说明是println!的问题，这刚开始就知道了println!是一个宏而不是函数，现在我们来将这个宏展开看看。

```rust
fn main() {
    let mut s = String::from("hello");
    let s1 = &mut s;
    println!("{}", s);
    println!("{}", s1);
}
```

```bash
cargo expand
```

```rust
#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
fn main() {
    let mut s = String::from("hello");
    let s1 = &mut s;	//s1 所有权开始
    {
        ::std::io::_print(::core::fmt::Arguments::new_v1(
            &["", "\n"],
            &[::core::fmt::ArgumentV1::new_display(&s)],
        ));
    };	//这里调用s的不可变引用和s1产生冲突，导致编译失败。
    {
        ::std::io::_print(::core::fmt::Arguments::new_v1(
            &["", "\n"],
            &[::core::fmt::ArgumentV1::new_display(&s1)],
        ));
    };	//s1 所有权结束。
}
```

可以看见println!其实是对s的一个不可变引用。但是由于s1的存在导致了，不能同时存在某个变量的可变引用和不可变引用，导致编译失败。

```rust
fn main() {
    let mut s = String::from("hello");
    let s1 = &mut s;
    println!("{}", s1);
    println!("{}", s);
}
```

```rust
#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
fn main() {
    let mut s = String::from("hello");
    let s1 = &mut s;	//s1 所有权开始
    {
        ::std::io::_print(::core::fmt::Arguments::new_v1(
            &["", "\n"],
            &[::core::fmt::ArgumentV1::new_display(&s1)],
        ));
    };	//s1 所有权结束。
    {
        ::std::io::_print(::core::fmt::Arguments::new_v1(
            &["", "\n"],
            &[::core::fmt::ArgumentV1::new_display(&s)],
        ));
    };	//由于后面的代码没有使用过s1，所以可以使用s的不可变引用。编译成功。
}
```

good！！！

# 更换参考书

https://course.rs/

因为之前的太旧了

## 复杂数据类型

### 字符串和切片

```rust
#![allow(unused)]
fn main() {
 let s = "中国人";
 let a = &s[0..2];	//会错误，因为是UTF-8字符，占用3个字节，取第一个汉字应该是[0..3]
 println!("{}",a);
}
```

```rust
fn main() {
    let mut s = String::from("hello world");

    let word = first_word(&s);	//不可变引用

    s.clear(); // error! clear调用的是可变引用

    println!("the first word is: {}", word);
}
fn first_word(s: &String) -> &str {
    &s[..1]
}
```

```rust
fn main() {
    let s1 = String::from("hello,");
    let s2 = String::from("world!");
    // 在下句中，s1的所有权被转移走了，因此后面不能再使用s1
    let s3 = s1 + &s2;
    assert_eq!(s3,"hello,world!");
    // 下面的语句如果去掉注释，就会报错
    // println!("{}",s1);
}

```

```rust
fn main() {
    let s1 = "hello";
    let s2 = String::from("rust");
    let s = format!("{} {}!", s1, s2);
    println!("{}", s);
}


```

### 元组





