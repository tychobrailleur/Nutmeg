use crate::chpp::model::StaffList;
use diesel::prelude::*;

pub fn save_staff(
    conn: &mut SqliteConnection,
    staff_data: &StaffList,
    for_team_id: u32,
    dl_id: i32,
) -> QueryResult<()> {
    use crate::db::schema::staff::dsl::*;

    if let Some(ref members) = staff_data.StaffMembers {
        for member in &members.staff {
            diesel::insert_into(staff)
                .values((
                    staff_id.eq(member.StaffId as i32),
                    team_id.eq(for_team_id as i32),
                    staff_type.eq(member.StaffType as i32),
                    staff_level.eq(member.StaffLevel as i32),
                    hired_date.eq(&member.HiredDate),
                    cost.eq(member.Cost as i32),
                    name.eq(&member.Name),
                    download_id.eq(dl_id),
                ))
                .execute(conn)?;
        }
    }

    Ok(())
}
