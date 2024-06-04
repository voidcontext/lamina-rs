use git2::Repository;

fn main() {
    let repo = Repository::open(".").unwrap();

    println!(
        "{:?}",
        repo.find_branch("main", git2::BranchType::Local)
            .unwrap()
            .get()
            .name()
    );
    println!(
        "{:?}",
        repo.find_reference("refs/tags/v0.2.0")
            .unwrap()
            .resolve()
            .unwrap()
            .target()
    );
    println!(
        "{:?}",
        repo.find_reference("refs/remotes/origin/HEAD")
            .unwrap()
            .kind()
    );
    let target = repo
        .find_reference("refs/tags/v0.2.0")
        .unwrap()
        .resolve()
        .unwrap()
        .target()
        .unwrap();
    println!("{:?}", repo.find_commit(target).unwrap().raw_header());
    // println!("{:?}", repo.find_reference("refs/tags/v0.2.1").unwrap().name().map(std::string::ToString::to_string));
}
