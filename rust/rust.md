```rust
use std::io; //导入库，use

fn main() {
    println!("Guess the number!");
    println!("Please input your guess.");

    let mut guess = String::new();	//可变变量，mut
    io::stdin().read_line(&mut guess)
        .expect("Failed to read line");

    println!("You guessed: {}", guess);
}
```

String::new中的::语法表明new是String类型的一个关联函数（associated function）。我们会针对类型本身来定义关联函数，比如本例中的String，而不会针对String的某个特定实例。关联函数在某些语言中也被称为静态方法（static method）。

```rust
pub fn read_line(&self, buf: &mut String) -> io::Result<usize> {
    self.lock().read_line(buf)
}
```

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

        let guess: u32 = match guess.trim().parse() {	//match匹配，返回值num
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

